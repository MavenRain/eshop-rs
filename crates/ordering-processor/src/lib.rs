//! Outbox publisher worker for the Ordering bounded context.
//!
//! Mirrors upstream eShop's `OrderingIntegrationEventService` /
//! background processor: it polls the ordering integration-event log
//! and forwards every [`EventState::NotPublished`] row onto an
//! [`EventBus`].
//!
//! # Pipeline
//!
//! For each cycle [`OrderProcessor::drain_once`] runs:
//!
//! 1. Open a transaction, [`retrieve_all_pending`] the
//!    `NotPublished` rows, commit.
//! 2. For each row, sequentially:
//!     1. Open a fresh transaction, mark the row
//!        [`EventState::InProgress`] and increment its `times_sent`,
//!        commit.
//!     2. Deserialize the JSON `content` column into an
//!        [`OrderingIntegrationEvent`] and publish it via
//!        [`EventBus::publish`].  The bus's [`Io`] is driven on a
//!        blocking thread (it owns its own runtime in the
//!        `RabbitMqEventBus` case) so it does not stall the async
//!        worker.
//!     3. Open a fresh transaction, mark the row
//!        [`EventState::Published`] on success or
//!        [`EventState::PublishedFailed`] on bus / deserialize error,
//!        commit.
//!
//! Per-row errors that originate inside the bus or the content column
//! (deserialization, transport) are absorbed into the
//! `PublishedFailed` row state; only persistence failures propagate
//! out of [`OrderProcessor::drain_once`].
//!
//! [`retrieve_all_pending`]: ordering_infrastructure::IntegrationEventLogService::retrieve_all_pending
//! [`EventState::NotPublished`]: ordering_infrastructure::EventState::NotPublished
//! [`EventState::InProgress`]: ordering_infrastructure::EventState::InProgress
//! [`EventState::Published`]: ordering_infrastructure::EventState::Published
//! [`EventState::PublishedFailed`]: ordering_infrastructure::EventState::PublishedFailed
//! [`Io`]: comp_cat_rs::effect::io::Io

pub mod error;

use std::sync::Arc;
use std::time::Duration;

use event_bus::EventBus;
use futures_lite::stream::StreamExt;
use ordering_infrastructure::{EventLog, IntegrationEventLogService};
use ordering_integration_events::OrderingIntegrationEvent;
use toasty::Db;

pub use error::Error;

/// Outbox publisher worker for the Ordering bounded context.
///
/// Generic over the concrete [`EventBus`] implementation so tests can
/// substitute an [`InMemoryEventBus`](event_bus::InMemoryEventBus) and
/// production wires up a [`RabbitMqEventBus`](event_bus_rabbitmq::RabbitMqEventBus).
///
/// `Db` is shared via [`Arc`] for the same FFI reason as
/// [`AppState`](ordering_api::AppState): toasty's [`Db::connection`]
/// takes `&self` and the worker's per-row processing opens fresh
/// connections.
pub struct OrderProcessor<B>
where
    B: EventBus<OrderingIntegrationEvent>,
{
    bus: B,
    db: Arc<Db>,
    poll_interval: Duration,
}

impl<B> OrderProcessor<B>
where
    B: EventBus<OrderingIntegrationEvent>,
{
    /// Construct from a bus, a shared toasty [`Db`], and the desired
    /// poll interval.  The poll interval is consulted by external
    /// drivers (e.g. an `eShop.AppHost` replacement) that schedule
    /// repeated calls to [`drain_once`](Self::drain_once); the worker
    /// itself does not loop.
    #[must_use]
    pub fn new(bus: B, db: Arc<Db>, poll_interval: Duration) -> Self {
        Self {
            bus,
            db,
            poll_interval,
        }
    }

    /// Borrow the configured poll interval.
    #[must_use]
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// Borrow the bus.  Exposed for tests and for external drivers
    /// that want to inspect the transport.
    #[must_use]
    pub fn bus(&self) -> &B {
        &self.bus
    }
}

impl<B> OrderProcessor<B>
where
    B: EventBus<OrderingIntegrationEvent> + Send + Sync + 'static,
{
    /// Process every currently-pending row once, in order, and return
    /// the number of rows attempted.
    ///
    /// # Errors
    /// Returns the first persistence error encountered in any per-row
    /// transition.  Bus and deserialization failures are absorbed into
    /// the row's `PublishedFailed` state and do not surface here.
    pub async fn drain_once(&self) -> Result<usize, Error> {
        let pending = self.fetch_pending().await?;
        let count = pending.len();
        // Sequential async iteration: `then` drives futures one at a
        // time and `collect` materializes the per-row results into a
        // `Vec<Result<(), Error>>`.  The trailing
        // `collect::<Result<Vec<()>, _>>` then short-circuits on the
        // first persistence error after every row has been attempted.
        let results: Vec<Result<(), Error>> = futures_lite::stream::iter(pending)
            .then(|log| async move { self.process_one(log).await })
            .collect()
            .await;
        results.into_iter().collect::<Result<Vec<()>, Error>>()?;
        Ok(count)
    }

    async fn fetch_pending(&self) -> Result<Vec<EventLog>, Error> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        let pending = IntegrationEventLogService::retrieve_all_pending(&mut tx).await?;
        tx.commit().await?;
        Ok(pending)
    }

    async fn process_one(&self, log: EventLog) -> Result<(), Error> {
        self.transition_to_in_progress(log.event_id()).await?;
        let publish_outcome = publish_log(&self.bus, &log).await;
        publish_outcome
            .as_ref()
            .err()
            .iter()
            .for_each(|err| eprintln!("publish failed for event {}: {err}", log.event_id()));
        let succeeded = publish_outcome.is_ok();
        if succeeded {
            self.transition_to_published(log.event_id()).await
        } else {
            self.transition_to_failed(log.event_id()).await
        }
    }

    async fn transition_to_in_progress(&self, event_id: uuid::Uuid) -> Result<(), Error> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        IntegrationEventLogService::mark_in_progress(&mut tx, event_id).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn transition_to_published(&self, event_id: uuid::Uuid) -> Result<(), Error> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        IntegrationEventLogService::mark_published(&mut tx, event_id).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn transition_to_failed(&self, event_id: uuid::Uuid) -> Result<(), Error> {
        let mut conn = self.db.connection().await?;
        let mut tx = conn.transaction().await?;
        IntegrationEventLogService::mark_failed(&mut tx, event_id).await?;
        tx.commit().await?;
        Ok(())
    }
}

/// Deserialize a single outbox row's `content` column into an
/// [`OrderingIntegrationEvent`] and publish it via `bus`.
///
/// The bus's [`Io`](comp_cat_rs::effect::io::Io) is driven on a
/// blocking thread because the production
/// [`RabbitMqEventBus`](event_bus_rabbitmq::RabbitMqEventBus)
/// implementation owns its own tokio runtime and would otherwise
/// stall the calling async runtime.
///
/// Exposed at the crate root so it can be tested independently of the
/// toasty-backed orchestration in [`OrderProcessor`].
///
/// # Errors
/// - [`Error::Json`] if the content column does not deserialize as an
///   [`OrderingIntegrationEvent`].
/// - [`Error::Worker`] if the blocking task panics or is cancelled.
/// - [`Error::Bus`] if the bus reports a transport failure.
pub async fn publish_log<B>(bus: &B, log: &EventLog) -> Result<(), Error>
where
    B: EventBus<OrderingIntegrationEvent> + Send + Sync + 'static,
{
    let event: OrderingIntegrationEvent = serde_json::from_str(log.content())?;
    let publish_io = bus.publish(event);
    let join = tokio::task::spawn_blocking(move || publish_io.run()).await;
    let bus_outcome = join.map_err(|e| Error::Worker {
        reason: format!("spawn_blocking join: {e}"),
    })?;
    bus_outcome.map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use event_bus::{InMemoryEventBus, IntegrationEvent};
    use ordering_infrastructure::EventState;
    use ordering_integration_events::{
        ORDER_SHIPPED, OrderShippedIntegrationEventPayload, OrderingIntegrationEvent,
    };
    use std::sync::mpsc::Receiver;
    use uuid::Uuid;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        match () {
            () if cond => Ok(()),
            () => Err(Error::Worker { reason: reason() }),
        }
    }

    fn shipped_event_log(order_id: Uuid) -> Result<EventLog, Error> {
        let event = OrderingIntegrationEvent::OrderShipped(
            OrderShippedIntegrationEventPayload::new(order_id),
        );
        let content = serde_json::to_string(&event)?;
        Ok(EventLog::new(
            Uuid::new_v4(),
            event.event_name().to_string(),
            Utc::now(),
            content,
            Uuid::new_v4(),
            EventState::NotPublished,
            0,
        ))
    }

    fn shipped_order_id(event: &OrderingIntegrationEvent) -> Option<Uuid> {
        match event {
            OrderingIntegrationEvent::OrderShipped(p) => Some(p.order_id()),
            OrderingIntegrationEvent::OrderStarted(_)
            | OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(_)
            | OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(_)
            | OrderingIntegrationEvent::OrderStatusChangedToPaid(_)
            | OrderingIntegrationEvent::OrderCancelled(_) => None,
        }
    }

    #[tokio::test]
    async fn publish_log_forwards_event_to_bus() -> Result<(), Error> {
        let (bus, rx): (InMemoryEventBus<OrderingIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let order_id = Uuid::new_v4();
        let log = shipped_event_log(order_id)?;
        publish_log(&bus, &log).await?;
        let received = rx.try_recv().map_err(|e| Error::Worker {
            reason: format!("recv: {e}"),
        })?;
        let actual = shipped_order_id(&received).unwrap_or_default();
        check(actual == order_id, || format!("got {actual}"))?;
        check(received.event_name() == ORDER_SHIPPED, || {
            format!("name {}", received.event_name())
        })
    }

    #[tokio::test]
    async fn publish_log_rejects_invalid_json() -> Result<(), Error> {
        let (bus, _rx): (InMemoryEventBus<OrderingIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let log = EventLog::new(
            Uuid::new_v4(),
            ORDER_SHIPPED.to_string(),
            Utc::now(),
            "{ not json".to_string(),
            Uuid::new_v4(),
            EventState::NotPublished,
            0,
        );
        let outcome = publish_log(&bus, &log).await;
        check(matches!(outcome, Err(Error::Json { .. })), || {
            format!("expected Error::Json, got {outcome:?}")
        })
    }

    #[tokio::test]
    async fn publish_log_rejects_unknown_event_name() -> Result<(), Error> {
        let (bus, _rx): (InMemoryEventBus<OrderingIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let log = EventLog::new(
            Uuid::new_v4(),
            "WhoKnowsIntegrationEvent".to_string(),
            Utc::now(),
            r#"{"event_name":"WhoKnowsIntegrationEvent"}"#.to_string(),
            Uuid::new_v4(),
            EventState::NotPublished,
            0,
        );
        let outcome = publish_log(&bus, &log).await;
        check(matches!(outcome, Err(Error::Json { .. })), || {
            format!("expected Error::Json, got {outcome:?}")
        })
    }
}

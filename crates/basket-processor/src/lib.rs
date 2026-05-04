//! Outbox publisher worker for the Basket bounded context.
//!
//! Mirror of `ordering-processor` and `catalog-processor`, scoped to
//! the basket's outbox
//! ([`basket::row::BasketIntegrationEventLogRow`]) and integration
//! event ([`BasketIntegrationEvent`]).  See `ordering-processor`'s
//! module docs for the full pipeline.

pub mod error;

use std::sync::Arc;
use std::time::Duration;

use basket::{EventLog, IntegrationEventLogService};
use basket_integration_events::BasketIntegrationEvent;
use event_bus::EventBus;
use futures_lite::stream::StreamExt;
use toasty::Db;

pub use error::Error;

/// Outbox publisher worker for the Basket bounded context.
pub struct BasketProcessor<B>
where
    B: EventBus<BasketIntegrationEvent>,
{
    bus: B,
    db: Arc<Db>,
    poll_interval: Duration,
}

impl<B> BasketProcessor<B>
where
    B: EventBus<BasketIntegrationEvent>,
{
    /// Construct from a bus, a shared toasty [`Db`], and the desired
    /// poll interval.  The poll interval is consulted by external
    /// drivers; the worker itself does not loop.
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

    /// Borrow the bus.
    #[must_use]
    pub fn bus(&self) -> &B {
        &self.bus
    }
}

impl<B> BasketProcessor<B>
where
    B: EventBus<BasketIntegrationEvent> + Send + Sync + 'static,
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

/// Deserialize a single outbox row's `content` column into a
/// [`BasketIntegrationEvent`] and publish it via `bus`.
///
/// The bus's [`Io`](comp_cat_rs::effect::io::Io) is driven on a
/// blocking thread because the production `RabbitMqEventBus` owns
/// its own tokio runtime; calling [`Io::run`](comp_cat_rs::effect::io::Io::run)
/// directly would stall the calling async runtime.
///
/// # Errors
/// - [`Error::Json`] if the content column does not deserialize.
/// - [`Error::Worker`] if the blocking task panics or is cancelled.
/// - [`Error::Bus`] if the bus reports a transport failure.
pub async fn publish_log<B>(bus: &B, log: &EventLog) -> Result<(), Error>
where
    B: EventBus<BasketIntegrationEvent> + Send + Sync + 'static,
{
    let event: BasketIntegrationEvent = serde_json::from_str(log.content())?;
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

    use basket::EventState;
    use basket_integration_events::{
        BasketIntegrationEvent, USER_CHECKOUT_ACCEPTED, UserCheckoutAcceptedIntegrationEventPayload,
    };
    use chrono::Utc;
    use event_bus::{InMemoryEventBus, IntegrationEvent};
    use ordering_integration_events::OrderStockItem;
    use std::sync::mpsc::Receiver;
    use uuid::Uuid;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Worker { reason: reason() })
        }
    }

    fn checkout_log(customer_id: Uuid) -> Result<EventLog, Error> {
        let event = BasketIntegrationEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(
                customer_id,
                Uuid::new_v4(),
                vec![OrderStockItem::new(42, 3)],
            ),
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

    fn customer_id_of(event: &BasketIntegrationEvent) -> Uuid {
        match event {
            BasketIntegrationEvent::UserCheckoutAccepted(p) => p.customer_id(),
        }
    }

    #[tokio::test]
    async fn publish_log_forwards_event_to_bus() -> Result<(), Error> {
        let (bus, rx): (InMemoryEventBus<BasketIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let customer_id = Uuid::new_v4();
        let log = checkout_log(customer_id)?;
        publish_log(&bus, &log).await?;
        let received = rx.try_recv().map_err(|e| Error::Worker {
            reason: format!("recv: {e}"),
        })?;
        check(customer_id_of(&received) == customer_id, || {
            format!("got {}", customer_id_of(&received))
        })?;
        check(received.event_name() == USER_CHECKOUT_ACCEPTED, || {
            format!("name {}", received.event_name())
        })
    }

    #[tokio::test]
    async fn publish_log_rejects_invalid_json() -> Result<(), Error> {
        let (bus, _rx): (InMemoryEventBus<BasketIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let log = EventLog::new(
            Uuid::new_v4(),
            USER_CHECKOUT_ACCEPTED.to_string(),
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
        let (bus, _rx): (InMemoryEventBus<BasketIntegrationEvent>, Receiver<_>) =
            InMemoryEventBus::new();
        let log = EventLog::new(
            Uuid::new_v4(),
            "UnknownIntegrationEvent".to_string(),
            Utc::now(),
            r#"{"event_name":"UnknownIntegrationEvent"}"#.to_string(),
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

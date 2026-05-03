//! Transactional outbox for the Catalog bounded context.
//!
//! Mirror of `ordering-infrastructure::integration_event_log`, scoped
//! to catalog's own [`CatalogIntegrationEventLogRow`] so the two
//! services own independent tables (mirrors upstream eShop's
//! per-service outbox convention).
//!
//! Callers stage events inside the same toasty
//! [`Transaction`](toasty::Transaction) that commits the aggregate
//! change; a separate worker drains pending rows and publishes through
//! the bus.  That worker lands with the eShop.AppHost replacement.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Error;
use crate::row::CatalogIntegrationEventLogRow;
use crate::time::{chrono_to_jiff, jiff_to_chrono};

/// State of an outbox row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventState {
    /// Not yet picked up by a publisher.
    NotPublished,
    /// A publisher has claimed the row and is attempting to publish.
    InProgress,
    /// Publisher confirmed publication.
    Published,
    /// Publisher reported a failure.
    PublishedFailed,
}

impl EventState {
    /// Persistence form (stable across schema versions).
    #[must_use]
    pub fn as_persisted(self) -> &'static str {
        match self {
            Self::NotPublished => "NotPublished",
            Self::InProgress => "InProgress",
            Self::Published => "Published",
            Self::PublishedFailed => "PublishedFailed",
        }
    }

    /// Parse the persisted form.
    ///
    /// # Errors
    /// Returns [`Error::InvalidPersistedValue`] if `s` is not one of the
    /// known states.
    pub fn parse_persisted(s: &str) -> Result<Self, Error> {
        match s {
            "NotPublished" => Ok(Self::NotPublished),
            "InProgress" => Ok(Self::InProgress),
            "Published" => Ok(Self::Published),
            "PublishedFailed" => Ok(Self::PublishedFailed),
            other => Err(Error::InvalidPersistedValue {
                context: "catalog_integration_event_logs.state".to_string(),
                value: other.to_string(),
            }),
        }
    }
}

/// Caller-supplied row to insert into the outbox.  Decoupled from the
/// `event_bus::IntegrationEvent` trait so the application controls
/// metadata generation and serialization shape.
#[derive(Debug, Clone)]
pub struct PendingEventLog {
    event_id: Uuid,
    event_type_name: String,
    creation_time: DateTime<Utc>,
    content: String,
    transaction_id: Uuid,
}

impl PendingEventLog {
    /// Construct.
    #[must_use]
    pub fn new(
        event_id: Uuid,
        event_type_name: String,
        creation_time: DateTime<Utc>,
        content: String,
        transaction_id: Uuid,
    ) -> Self {
        Self {
            event_id,
            event_type_name,
            creation_time,
            content,
            transaction_id,
        }
    }

    /// Event identifier.
    #[must_use]
    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Event type name.
    #[must_use]
    pub fn event_type_name(&self) -> &str {
        &self.event_type_name
    }

    /// Creation timestamp.
    #[must_use]
    pub fn creation_time(&self) -> DateTime<Utc> {
        self.creation_time
    }

    /// Pre-serialized JSON content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Transaction id.
    #[must_use]
    pub fn transaction_id(&self) -> Uuid {
        self.transaction_id
    }
}

/// A loaded outbox row.
#[derive(Debug, Clone)]
pub struct EventLog {
    event_id: Uuid,
    event_type_name: String,
    creation_time: DateTime<Utc>,
    content: String,
    transaction_id: Uuid,
    state: EventState,
    times_sent: i64,
}

impl EventLog {
    /// Event identifier.
    #[must_use]
    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    /// Event type name.
    #[must_use]
    pub fn event_type_name(&self) -> &str {
        &self.event_type_name
    }

    /// Creation timestamp.
    #[must_use]
    pub fn creation_time(&self) -> DateTime<Utc> {
        self.creation_time
    }

    /// Pre-serialized JSON content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Transaction id.
    #[must_use]
    pub fn transaction_id(&self) -> Uuid {
        self.transaction_id
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> EventState {
        self.state
    }

    /// Number of publish attempts so far.
    #[must_use]
    pub fn times_sent(&self) -> i64 {
        self.times_sent
    }
}

fn row_to_log(row: CatalogIntegrationEventLogRow) -> Result<EventLog, Error> {
    Ok(EventLog {
        event_id: row.event_id,
        event_type_name: row.event_type_name,
        creation_time: jiff_to_chrono(row.creation_time)?,
        content: row.content,
        transaction_id: row.transaction_id,
        state: EventState::parse_persisted(&row.state)?,
        times_sent: row.times_sent,
    })
}

/// Outbox service.
pub struct IntegrationEventLogService;

impl IntegrationEventLogService {
    /// Insert a new outbox row in the [`EventState::NotPublished`] state.
    /// Caller runs this inside the transaction that commits the
    /// aggregate change.
    ///
    /// # Errors
    /// Propagates toasty and time-conversion errors.
    pub async fn save_event<E>(executor: &mut E, log: PendingEventLog) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        toasty::create!(CatalogIntegrationEventLogRow {
            event_id: log.event_id(),
            event_type_name: log.event_type_name().to_string(),
            state: EventState::NotPublished.as_persisted().to_string(),
            times_sent: 0_i64,
            creation_time: chrono_to_jiff(log.creation_time())?,
            content: log.content().to_string(),
            transaction_id: log.transaction_id(),
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Insert a batch of outbox rows in a single transaction.  Identical
    /// semantics to calling [`save_event`](Self::save_event) once per row.
    ///
    /// The internal sequential await is the localized FFI exception
    /// permitted because each iteration consumes `&mut executor` (toasty's
    /// `Executor` requires `&mut self`).  Concentrating the loop here
    /// keeps the API-layer call sites combinator-only.
    ///
    /// # Errors
    /// Propagates toasty and time-conversion errors; the first error
    /// short-circuits the batch.
    pub async fn save_events_batch<E>(
        executor: &mut E,
        logs: Vec<PendingEventLog>,
    ) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        for log in logs {
            Self::save_event(executor, log).await?;
        }
        Ok(())
    }

    /// Retrieve every [`EventState::NotPublished`] row attached to
    /// `transaction_id`.
    ///
    /// # Errors
    /// Propagates toasty and time-conversion errors.
    pub async fn retrieve_pending<E>(
        executor: &mut E,
        transaction_id: Uuid,
    ) -> Result<Vec<EventLog>, Error>
    where
        E: toasty::Executor,
    {
        let target_state = EventState::NotPublished.as_persisted().to_string();
        let rows: Vec<CatalogIntegrationEventLogRow> = toasty::query!(
            CatalogIntegrationEventLogRow FILTER .transaction_id == #transaction_id AND .state == #target_state
        )
        .exec(executor)
        .await?;
        rows.into_iter().map(row_to_log).collect()
    }

    /// Mark a single row as [`EventState::Published`].
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn mark_published<E>(executor: &mut E, event_id: Uuid) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        Self::set_state(executor, event_id, EventState::Published, false).await
    }

    /// Mark a single row as [`EventState::InProgress`] and increment
    /// its `times_sent` counter.
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn mark_in_progress<E>(executor: &mut E, event_id: Uuid) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        Self::set_state(executor, event_id, EventState::InProgress, true).await
    }

    /// Mark a single row as [`EventState::PublishedFailed`].
    ///
    /// # Errors
    /// Propagates toasty errors.
    pub async fn mark_failed<E>(executor: &mut E, event_id: Uuid) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        Self::set_state(executor, event_id, EventState::PublishedFailed, false).await
    }

    async fn set_state<E>(
        executor: &mut E,
        event_id: Uuid,
        state: EventState,
        bump_times_sent: bool,
    ) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        let row = CatalogIntegrationEventLogRow::get_by_event_id(executor, &event_id).await?;
        let new_times_sent = if bump_times_sent {
            row.times_sent.saturating_add(1)
        } else {
            row.times_sent
        };
        CatalogIntegrationEventLogRow::update_by_event_id(event_id)
            .state(state.as_persisted().to_string())
            .times_sent(new_times_sent)
            .exec(executor)
            .await?;
        Ok(())
    }
}

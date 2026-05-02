//! Integration event metadata and trait.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

/// Common metadata carried on every integration event: a unique id and a
/// creation timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntegrationEventMetadata {
    id: Uuid,
    creation_date: DateTime<Utc>,
}

impl IntegrationEventMetadata {
    /// Capture a fresh id and the current UTC timestamp.
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            id: Uuid::new_v4(),
            creation_date: Utc::now(),
        }
    }

    /// Construct from raw components (used when rehydrating from persistence
    /// or the wire).
    #[must_use]
    pub fn from_parts(id: Uuid, creation_date: DateTime<Utc>) -> Self {
        Self { id, creation_date }
    }

    /// Event identifier.
    #[must_use]
    pub fn id(self) -> Uuid {
        self.id
    }

    /// UTC timestamp at which the event was created.
    #[must_use]
    pub fn creation_date(self) -> DateTime<Utc> {
        self.creation_date
    }
}

impl Default for IntegrationEventMetadata {
    fn default() -> Self {
        Self::fresh()
    }
}

/// Trait every application integration event implements.
///
/// Implementations are typically a sum type with one variant per concrete
/// event.  Each variant maps to a stable `event_name` string used as the
/// routing key on the wire.  All possible names are returned by
/// [`IntegrationEvent::all_event_names`] so subscribers can bind their
/// queues at startup.
///
/// `Serialize` + `DeserializeOwned` are required so a concrete bus can
/// move events to and from byte payloads.  We recommend a `serde` tagged
/// enum (`#[serde(tag = "event_name")]`) so the wire format is
/// self-describing.
pub trait IntegrationEvent: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Wire name for this specific event instance, used as the routing key.
    fn event_name(&self) -> &'static str;

    /// All possible event names.  Subscribers bind their queue to each.
    fn all_event_names() -> &'static [&'static str];
}

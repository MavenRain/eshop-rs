//! Crate error type for the ordering outbox publisher worker.
//!
//! Layered like other workspace error enums:
//!
//! - [`Toasty`](Error::Toasty) for raw toasty driver failures.
//! - [`Infrastructure`](Error::Infrastructure) for any
//!   `ordering-infrastructure` error surfaced by the outbox service or
//!   repositories.
//! - [`Bus`](Error::Bus) for `event-bus` publish failures.
//! - [`Json`](Error::Json) for content-column deserialization failures.
//! - [`Worker`](Error::Worker) for `tokio::spawn_blocking` join errors
//!   and other glue failures.

/// Errors produced by the outbox publisher worker.
#[derive(Debug)]
pub enum Error {
    /// Raw toasty driver failure.
    Toasty { reason: String },
    /// Any `ordering-infrastructure` error.
    Infrastructure { reason: String },
    /// `event-bus` publish failure.
    Bus { reason: String },
    /// Outbox content failed to deserialize as an integration event.
    Json { reason: String },
    /// Worker glue failure (e.g., `spawn_blocking` join error).
    Worker { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::Infrastructure { reason } => write!(f, "infrastructure: {reason}"),
            Self::Bus { reason } => write!(f, "bus: {reason}"),
            Self::Json { reason } => write!(f, "json: {reason}"),
            Self::Worker { reason } => write!(f, "worker: {reason}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<toasty::Error> for Error {
    fn from(e: toasty::Error) -> Self {
        Self::Toasty {
            reason: e.to_string(),
        }
    }
}

impl From<ordering_infrastructure::Error> for Error {
    fn from(e: ordering_infrastructure::Error) -> Self {
        Self::Infrastructure {
            reason: e.to_string(),
        }
    }
}

impl From<event_bus::Error> for Error {
    fn from(e: event_bus::Error) -> Self {
        Self::Bus {
            reason: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json {
            reason: e.to_string(),
        }
    }
}

//! Crate error type for the basket outbox publisher worker.

/// Errors produced by the basket outbox publisher worker.
#[derive(Debug)]
pub enum Error {
    /// Raw toasty driver failure.
    Toasty { reason: String },
    /// Any `basket` crate error (persistence, mapping, validation).
    Basket { reason: String },
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
            Self::Basket { reason } => write!(f, "basket: {reason}"),
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

impl From<basket::Error> for Error {
    fn from(e: basket::Error) -> Self {
        Self::Basket {
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

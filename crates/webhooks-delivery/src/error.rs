//! Crate error type for the webhooks delivery worker.

/// Errors produced by the webhooks delivery worker.
#[derive(Debug)]
pub enum Error {
    /// Toasty driver failure surfaced from a handler body.
    Toasty { reason: String },
    /// `webhooks` crate error (subscription persistence / mapping).
    Webhooks { reason: String },
    /// Outbound payload failed to serialize as JSON.
    Json { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::Webhooks { reason } => write!(f, "webhooks: {reason}"),
            Self::Json { reason } => write!(f, "json: {reason}"),
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

impl From<webhooks::Error> for Error {
    fn from(e: webhooks::Error) -> Self {
        Self::Webhooks {
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

impl From<Error> for event_bus::Error {
    fn from(e: Error) -> Self {
        Self::Subscribe {
            reason: e.to_string(),
        }
    }
}

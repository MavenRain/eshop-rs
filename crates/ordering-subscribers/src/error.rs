//! Crate error type for ordering-side integration event handlers.

/// Errors produced by ordering-side integration event handlers.
#[derive(Debug)]
pub enum Error {
    /// Toasty driver failure surfaced from a handler body.
    Toasty { reason: String },
    /// Any `ordering-domain` error surfaced from a handler body.
    Domain { reason: String },
    /// Any `ordering-infrastructure` error surfaced from a handler
    /// body.
    Infrastructure { reason: String },
    /// Outbox content serialization failure.
    Json { reason: String },
    /// Internal-state failure surfaced from a handler body.
    Handler { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::Domain { reason } => write!(f, "domain: {reason}"),
            Self::Infrastructure { reason } => write!(f, "infrastructure: {reason}"),
            Self::Json { reason } => write!(f, "json: {reason}"),
            Self::Handler { reason } => write!(f, "handler: {reason}"),
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

impl From<ordering_domain::Error> for Error {
    fn from(e: ordering_domain::Error) -> Self {
        Self::Domain {
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

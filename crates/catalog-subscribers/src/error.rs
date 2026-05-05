//! Crate error type for catalog-side integration event handlers.

/// Errors produced by catalog-side integration event handlers.
#[derive(Debug)]
pub enum Error {
    /// Toasty driver failure surfaced from a handler body.
    Toasty { reason: String },
    /// Any `catalog` crate error surfaced from a handler body.
    Catalog { reason: String },
    /// Internal-state failure surfaced from a handler body.
    Handler { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::Catalog { reason } => write!(f, "catalog: {reason}"),
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

impl From<catalog::Error> for Error {
    fn from(e: catalog::Error) -> Self {
        Self::Catalog {
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

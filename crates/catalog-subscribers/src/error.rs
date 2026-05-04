//! Crate error type for catalog-side integration event handlers.
//!
//! For the wire-validation slice the handlers do not yet touch
//! catalog persistence, so the variant set is small.  Once the
//! handlers grow real bodies (stock decrement, etc.) they will gain
//! `Catalog` and `Toasty` variants alongside.

/// Errors produced by catalog-side integration event handlers.
#[derive(Debug)]
pub enum Error {
    /// Internal-state failure surfaced from a handler body.
    Handler { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Handler { reason } => write!(f, "handler: {reason}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<Error> for event_bus::Error {
    fn from(e: Error) -> Self {
        Self::Subscribe {
            reason: e.to_string(),
        }
    }
}

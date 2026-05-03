//! Crate error type.

/// Errors produced by `ordering-infrastructure` operations.
#[derive(Debug)]
pub enum Error {
    /// Failure inside toasty.
    Toasty { reason: String },
    /// Domain validation rejected a value loaded from the database.
    Domain { reason: String },
    /// Timestamp could not be converted between chrono and jiff.
    TimeConversion { reason: String },
    /// JSON serialization or deserialization failed.
    Json { reason: String },
    /// A persisted column held a value the loader could not interpret.
    InvalidPersistedValue { context: String, value: String },
    /// Numeric out-of-range for the target type.
    NumericRange { context: String },
    /// A domain invariant was violated; carries a human-readable reason.
    InvariantViolated { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Toasty { reason } => write!(f, "toasty error: {reason}"),
            Self::Domain { reason } => write!(f, "domain validation: {reason}"),
            Self::TimeConversion { reason } => write!(f, "time conversion: {reason}"),
            Self::Json { reason } => write!(f, "json error: {reason}"),
            Self::InvalidPersistedValue { context, value } => {
                write!(f, "invalid persisted value at {context}: {value}")
            }
            Self::NumericRange { context } => write!(f, "numeric out of range at {context}"),
            Self::InvariantViolated { reason } => write!(f, "invariant violated: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json {
            reason: e.to_string(),
        }
    }
}

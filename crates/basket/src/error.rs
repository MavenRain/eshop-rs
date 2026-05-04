//! Crate error type for the Basket bounded context.
//!
//! Layered like other workspace error enums:
//!
//! - **Domain validation**: `EmptyString`, `NegativePrice`, `ZeroQuantity`,
//!   `InvariantViolated`.
//! - **Persistence**: `Toasty`, `NumericRange`, `InvalidPersistedValue`.
//! - **HTTP**: `Validation` (400), `NotFound` (404), `Json` (400).
//!
//! [`PartialEq`] / [`Eq`] hold across every variant for test ergonomics.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors produced by Basket operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Construction of a non-empty string failed because the input was empty.
    EmptyString { field: &'static str },
    /// A persisted price was negative.
    NegativePrice,
    /// A `Quantity` constructor received zero.
    ZeroQuantity,
    /// A general invariant was violated; the body explains.
    InvariantViolated { reason: String },
    /// Toasty driver-level failure.
    Toasty { reason: String },
    /// A persisted column held a value the loader could not interpret.
    InvalidPersistedValue { context: String, value: String },
    /// Numeric out-of-range when narrowing a persisted column.
    NumericRange { context: String },
    /// Request body or path parameter failed validation.
    Validation { reason: String },
    /// The requested resource does not exist.
    NotFound { reason: String },
    /// JSON serialization or deserialization failed.
    Json { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString { field } => write!(f, "empty string for {field}"),
            Self::NegativePrice => write!(f, "negative price"),
            Self::ZeroQuantity => write!(f, "zero quantity"),
            Self::InvariantViolated { reason } => write!(f, "invariant violated: {reason}"),
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::InvalidPersistedValue { context, value } => {
                write!(f, "invalid persisted value at {context}: {value}")
            }
            Self::NumericRange { context } => write!(f, "numeric range: {context}"),
            Self::Validation { reason } => write!(f, "validation: {reason}"),
            Self::NotFound { reason } => write!(f, "not found: {reason}"),
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

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json {
            reason: e.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::EmptyString { .. }
            | Self::NegativePrice
            | Self::ZeroQuantity
            | Self::InvariantViolated { .. }
            | Self::Validation { .. }
            | Self::Json { .. } => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Toasty { .. }
            | Self::InvalidPersistedValue { .. }
            | Self::NumericRange { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn status_for(error: Error) -> StatusCode {
        error.into_response().status()
    }

    #[test]
    fn empty_string_maps_to_400() -> Result<(), Error> {
        let actual = status_for(Error::EmptyString { field: "x" });
        check(actual == StatusCode::BAD_REQUEST, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn not_found_maps_to_404() -> Result<(), Error> {
        let actual = status_for(Error::NotFound {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::NOT_FOUND, || format!("got {actual}"))
    }

    #[test]
    fn toasty_maps_to_500() -> Result<(), Error> {
        let actual = status_for(Error::Toasty {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::INTERNAL_SERVER_ERROR, || {
            format!("got {actual}")
        })
    }
}

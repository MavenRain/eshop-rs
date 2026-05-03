//! Crate error type for the Catalog bounded context.
//!
//! Covers domain validation (constructor invariants on
//! [`CatalogItem`](crate::item::CatalogItem) etc.), persistence
//! (toasty-backed repositories), and the HTTP API
//! (axum [`IntoResponse`] mapping).  Variants are layered:
//!
//! - **Domain**: `EmptyString`, `NegativePrice`, `OutOfStock`,
//!   `RestockExceedsMax`, `InitialStockExceedsMax`, `StockOverflow`,
//!   `InvariantViolated`.
//! - **Persistence**: `Toasty`, `NumericRange`,
//!   `InvalidPersistedValue`, `TimeConversion`.
//! - **HTTP**: `Validation` (400), `NotFound` (404), `Json` (400).
//!
//! [`PartialEq`] / [`Eq`] hold across every variant for test ergonomics.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors produced by Catalog operations (domain + persistence + HTTP).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Construction of a non-empty string failed because the input was empty.
    EmptyString { field: &'static str },
    /// A [`Price`](crate::money::Price) was constructed with a negative amount.
    NegativePrice,
    /// [`CatalogItem::remove_stock`](crate::item::CatalogItem::remove_stock)
    /// was called when [`Stock`](crate::stock::Stock) was already zero.
    OutOfStock { item: String },
    /// `RestockThreshold` exceeded `MaxStockThreshold` at construction.
    RestockExceedsMax { restock: u32, max: u32 },
    /// Initial `Stock` exceeded `MaxStockThreshold` at construction.
    InitialStockExceedsMax { stock: u32, max: u32 },
    /// Stock arithmetic overflowed `u32`.
    StockOverflow,
    /// A domain invariant was violated; carries a human-readable reason.
    InvariantViolated { reason: String },
    /// Failure inside toasty.
    Toasty { reason: String },
    /// Numeric out-of-range for the target type when reading from a row.
    NumericRange { context: String },
    /// A persisted column held a value the loader could not interpret.
    InvalidPersistedValue { context: String, value: String },
    /// Timestamp could not be converted between chrono and jiff.
    TimeConversion { reason: String },
    /// Request body or query parameter failed validation.
    Validation { reason: String },
    /// The requested resource does not exist.
    NotFound { reason: String },
    /// JSON serialization or deserialization failed.
    Json { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString { field } => write!(f, "{field} must not be empty"),
            Self::NegativePrice => write!(f, "price must not be negative"),
            Self::OutOfStock { item } => {
                write!(f, "empty stock, product item {item} is sold out")
            }
            Self::RestockExceedsMax { restock, max } => write!(
                f,
                "restock threshold {restock} exceeds max stock threshold {max}"
            ),
            Self::InitialStockExceedsMax { stock, max } => {
                write!(f, "initial stock {stock} exceeds max stock threshold {max}")
            }
            Self::StockOverflow => write!(f, "stock arithmetic overflowed"),
            Self::InvariantViolated { reason } => write!(f, "invariant violated: {reason}"),
            Self::Toasty { reason } => write!(f, "toasty error: {reason}"),
            Self::NumericRange { context } => write!(f, "numeric out of range at {context}"),
            Self::InvalidPersistedValue { context, value } => {
                write!(f, "invalid persisted value at {context}: {value}")
            }
            Self::TimeConversion { reason } => write!(f, "time conversion: {reason}"),
            Self::Validation { reason } => write!(f, "validation: {reason}"),
            Self::NotFound { reason } => write!(f, "not found: {reason}"),
            Self::Json { reason } => write!(f, "json: {reason}"),
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
            // Domain validation surfaced from a request body.
            Self::EmptyString { .. }
            | Self::NegativePrice
            | Self::OutOfStock { .. }
            | Self::RestockExceedsMax { .. }
            | Self::InitialStockExceedsMax { .. }
            | Self::StockOverflow
            | Self::InvariantViolated { .. }
            | Self::Validation { .. }
            | Self::Json { .. } => StatusCode::BAD_REQUEST,
            // Lookup misses.
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            // Anything that should not have been visible at the request
            // boundary (database / clock / persisted-value bugs).
            Self::Toasty { .. }
            | Self::NumericRange { .. }
            | Self::InvalidPersistedValue { .. }
            | Self::TimeConversion { .. } => StatusCode::INTERNAL_SERVER_ERROR,
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
    fn validation_maps_to_bad_request() -> Result<(), Error> {
        let actual = status_for(Error::Validation {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::BAD_REQUEST, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn empty_string_maps_to_bad_request() -> Result<(), Error> {
        let actual = status_for(Error::EmptyString { field: "x" });
        check(actual == StatusCode::BAD_REQUEST, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn out_of_stock_maps_to_bad_request() -> Result<(), Error> {
        let actual = status_for(Error::OutOfStock {
            item: "x".to_string(),
        });
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

    #[test]
    fn time_conversion_maps_to_500() -> Result<(), Error> {
        let actual = status_for(Error::TimeConversion {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::INTERNAL_SERVER_ERROR, || {
            format!("got {actual}")
        })
    }
}

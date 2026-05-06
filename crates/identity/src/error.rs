//! Crate error type for the Identity bounded context.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors produced by Identity operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Construction of a non-empty string failed because the input was empty.
    EmptyString { field: &'static str },
    /// `Email` rejected an input lacking the minimal shape (`@`).
    MalformedEmail { value: String },
    /// Password hashing failed.
    PasswordHash { reason: String },
    /// Bearer credentials did not match a stored hash.
    InvalidCredentials,
    /// Toasty driver-level failure.
    Toasty { reason: String },
    /// A persisted column held a value the loader could not interpret.
    InvalidPersistedValue { context: String, value: String },
    /// Timestamp could not be converted between chrono and jiff.
    TimeConversion { reason: String },
    /// JWT issuance failure.
    Token { reason: String },
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
            Self::MalformedEmail { value } => write!(f, "malformed email: {value}"),
            Self::PasswordHash { reason } => write!(f, "password hash: {reason}"),
            Self::InvalidCredentials => write!(f, "invalid credentials"),
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::InvalidPersistedValue { context, value } => {
                write!(f, "invalid persisted value at {context}: {value}")
            }
            Self::TimeConversion { reason } => write!(f, "time conversion: {reason}"),
            Self::Token { reason } => write!(f, "token: {reason}"),
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

impl From<identity_tokens::Error> for Error {
    fn from(e: identity_tokens::Error) -> Self {
        Self::Token {
            reason: e.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::EmptyString { .. }
            | Self::MalformedEmail { .. }
            | Self::Validation { .. }
            | Self::Json { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::PasswordHash { .. }
            | Self::Toasty { .. }
            | Self::InvalidPersistedValue { .. }
            | Self::TimeConversion { .. }
            | Self::Token { .. } => StatusCode::INTERNAL_SERVER_ERROR,
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
    fn invalid_credentials_maps_to_401() -> Result<(), Error> {
        let actual = status_for(Error::InvalidCredentials);
        check(actual == StatusCode::UNAUTHORIZED, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn malformed_email_maps_to_400() -> Result<(), Error> {
        let actual = status_for(Error::MalformedEmail {
            value: "x".to_string(),
        });
        check(actual == StatusCode::BAD_REQUEST, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn password_hash_maps_to_500() -> Result<(), Error> {
        let actual = status_for(Error::PasswordHash {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::INTERNAL_SERVER_ERROR, || {
            format!("got {actual}")
        })
    }
}

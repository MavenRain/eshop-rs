//! Crate error type plus its [`axum::response::IntoResponse`] mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors produced by the Ordering API.
#[derive(Debug)]
pub enum Error {
    /// Request body or path parameter failed validation.
    Validation { reason: String },
    /// The requested order does not exist.
    NotFound { reason: String },
    /// Caller is authenticated but not authorized to access this order.
    Forbidden { reason: String },
    /// Domain rejected the operation (e.g., illegal status transition).
    Domain { reason: String },
    /// Persistence layer failure.
    Infrastructure { reason: String },
    /// JSON serialization or deserialization failed.
    Json { reason: String },
    /// Toasty driver failure.
    Toasty { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Validation { reason } => write!(f, "validation: {reason}"),
            Self::NotFound { reason } => write!(f, "not found: {reason}"),
            Self::Forbidden { reason } => write!(f, "forbidden: {reason}"),
            Self::Domain { reason } => write!(f, "domain: {reason}"),
            Self::Infrastructure { reason } => write!(f, "infrastructure: {reason}"),
            Self::Json { reason } => write!(f, "json: {reason}"),
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

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

impl From<toasty::Error> for Error {
    fn from(e: toasty::Error) -> Self {
        Self::Toasty {
            reason: e.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Validation { .. } | Self::Domain { .. } | Self::Json { .. } => {
                StatusCode::BAD_REQUEST
            }
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::Infrastructure { .. } | Self::Toasty { .. } => StatusCode::INTERNAL_SERVER_ERROR,
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
    fn domain_maps_to_bad_request() -> Result<(), Error> {
        let actual = status_for(Error::Domain {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::BAD_REQUEST, || {
            format!("got {actual}")
        })
    }

    #[test]
    fn json_maps_to_bad_request() -> Result<(), Error> {
        let actual = status_for(Error::Json {
            reason: "x".to_string(),
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
    fn forbidden_maps_to_403() -> Result<(), Error> {
        let actual = status_for(Error::Forbidden {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::FORBIDDEN, || format!("got {actual}"))
    }

    #[test]
    fn infrastructure_maps_to_500() -> Result<(), Error> {
        let actual = status_for(Error::Infrastructure {
            reason: "x".to_string(),
        });
        check(actual == StatusCode::INTERNAL_SERVER_ERROR, || {
            format!("got {actual}")
        })
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

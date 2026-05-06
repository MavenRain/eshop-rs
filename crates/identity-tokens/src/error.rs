//! Crate error type for identity-tokens.

/// Errors produced by JWT issuance and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// `jsonwebtoken` could not encode or decode the token.
    Jwt { reason: String },
    /// A claims value (typically `sub`) was not a valid `Uuid`.
    InvalidClaim { field: &'static str, reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Jwt { reason } => write!(f, "jwt: {reason}"),
            Self::InvalidClaim { field, reason } => {
                write!(f, "invalid claim {field}: {reason}")
            }
        }
    }
}

impl core::error::Error for Error {}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Jwt {
            reason: e.to_string(),
        }
    }
}

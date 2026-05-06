//! Token claims plus a typed view of them.
//!
//! The wire shape is `Claims` (serde-friendly, primitive fields).
//! Once a token is decoded, `claims_to_principal` projects the
//! claims into a [`Principal`] with newtyped values that callers
//! can hand to repositories.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

/// Wire-form JWT claims.
///
/// `sub` carries the user id as a string (Uuid).  `email` and
/// `name` are denormalized snapshots so consumers don't need a
/// directory lookup for common operations (logging, hello-world
/// endpoints).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub exp: i64,
    pub iat: i64,
}

/// Typed view of a successfully validated [`Claims`] payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    user_id: Uuid,
    email: String,
    name: String,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl Principal {
    /// Authenticated user id.
    #[must_use]
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// User email (denormalized snapshot at issuance time).
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// User display name (denormalized snapshot at issuance time).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `iat`.
    #[must_use]
    pub fn issued_at(&self) -> DateTime<Utc> {
        self.issued_at
    }

    /// `exp`.
    #[must_use]
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
}

/// Project a wire-form [`Claims`] payload into a [`Principal`].
///
/// # Errors
/// - [`Error::InvalidClaim`] if `sub` is not a valid `Uuid`, or if
///   `iat` / `exp` are outside chrono's representable range.
pub fn claims_to_principal(claims: &Claims) -> Result<Principal, Error> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| Error::InvalidClaim {
        field: "sub",
        reason: e.to_string(),
    })?;
    let issued_at =
        Utc.timestamp_opt(claims.iat, 0)
            .single()
            .ok_or_else(|| Error::InvalidClaim {
                field: "iat",
                reason: format!("seconds {} not representable", claims.iat),
            })?;
    let expires_at =
        Utc.timestamp_opt(claims.exp, 0)
            .single()
            .ok_or_else(|| Error::InvalidClaim {
                field: "exp",
                reason: format!("seconds {} not representable", claims.exp),
            })?;
    Ok(Principal {
        user_id,
        email: claims.email.clone(),
        name: claims.name.clone(),
        issued_at,
        expires_at,
    })
}

//! Response DTOs serialized to JSON response bodies.

use serde::Serialize;
use uuid::Uuid;

/// `POST /api/identity/register` response body.  Carries only the
/// minted user id; the caller follows up with `/api/identity/login`
/// to receive credentials.
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredUserResponse {
    id: Uuid,
}

impl RegisteredUserResponse {
    /// Wrap a fresh user id for the JSON response.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

/// `POST /api/identity/login` response body.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    access_token: String,
    token_type: &'static str,
    expires_in_seconds: i64,
}

impl LoginResponse {
    /// Construct a bearer-token response.
    #[must_use]
    pub fn bearer(access_token: String, expires_in_seconds: i64) -> Self {
        Self {
            access_token,
            token_type: "Bearer",
            expires_in_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::error::Error;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    #[test]
    fn login_response_serializes_with_bearer_type() -> Result<(), Error> {
        let response = LoginResponse::bearer("token-bytes".to_string(), 3600);
        let body = serde_json::to_string(&response)?;
        check(body.contains("\"token_type\":\"Bearer\""), || {
            format!("body {body}")
        })?;
        check(body.contains("\"access_token\":\"token-bytes\""), || {
            format!("body {body}")
        })?;
        check(body.contains("\"expires_in_seconds\":3600"), || {
            format!("body {body}")
        })
    }

    #[test]
    fn registered_user_response_carries_id() -> Result<(), Error> {
        let id = Uuid::new_v4();
        let response = RegisteredUserResponse::new(id);
        let body = serde_json::to_string(&response)?;
        check(body.contains(&id.to_string()), || format!("body {body}"))
    }
}

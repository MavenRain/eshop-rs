//! JWT issuance.

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use uuid::Uuid;

use crate::claims::Claims;
use crate::error::Error;

/// Configuration for JWT issuance.  `secret` is the HMAC key shared
/// with every consumer that validates tokens.
#[derive(Debug, Clone)]
pub struct IssuerConfig {
    secret: Vec<u8>,
    ttl: Duration,
}

impl IssuerConfig {
    /// Construct from a shared secret and a token TTL.
    #[must_use]
    pub fn new(secret: Vec<u8>, ttl: Duration) -> Self {
        Self { secret, ttl }
    }

    /// Borrow the HMAC secret.
    #[must_use]
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }

    /// Token lifetime.
    #[must_use]
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

/// Issue a signed JWT for `user_id` carrying `email` + `name`.
///
/// `now` is supplied so callers (and tests) can pin the issuance
/// timestamp; the `exp` claim is `now + config.ttl()`.
///
/// # Errors
/// Returns [`Error::Jwt`] if the underlying `jsonwebtoken` encode
/// step fails.
pub fn issue(
    config: &IssuerConfig,
    user_id: Uuid,
    email: &str,
    name: &str,
    now: DateTime<Utc>,
) -> Result<String, Error> {
    let exp = now + config.ttl();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        name: name.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret()),
    )?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::validator::{ValidatorConfig, validate};

    #[derive(Debug)]
    struct TestError(String);

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl core::error::Error for TestError {}

    impl From<Error> for TestError {
        fn from(e: Error) -> Self {
            Self(e.to_string())
        }
    }

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        if cond {
            Ok(())
        } else {
            Err(TestError(reason()))
        }
    }

    #[test]
    fn issue_then_validate_round_trips() -> Result<(), TestError> {
        let config = IssuerConfig::new(b"super-secret".to_vec(), Duration::hours(1));
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let token = issue(&config, user_id, "alice@example.test", "Alice", now)?;
        let validator = ValidatorConfig::new(config.secret().to_vec());
        let principal = validate(&validator, &token)?;
        check(principal.user_id() == user_id, || {
            format!("user_id {}", principal.user_id())
        })?;
        check(principal.email() == "alice@example.test", || {
            format!("email {}", principal.email())
        })?;
        check(principal.name() == "Alice", || {
            format!("name {}", principal.name())
        })
    }

    #[test]
    fn validate_rejects_token_signed_with_different_secret() -> Result<(), TestError> {
        let issuer = IssuerConfig::new(b"original".to_vec(), Duration::hours(1));
        let token = issue(
            &issuer,
            Uuid::new_v4(),
            "alice@example.test",
            "Alice",
            Utc::now(),
        )?;
        let attacker = ValidatorConfig::new(b"wrong-secret".to_vec());
        let outcome = validate(&attacker, &token);
        check(matches!(outcome, Err(Error::Jwt { .. })), || {
            format!("got {outcome:?}")
        })
    }
}

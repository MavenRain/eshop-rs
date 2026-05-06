//! JWT validation.

use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::claims::{Claims, Principal, claims_to_principal};
use crate::error::Error;

/// Configuration for JWT validation.  Holds the shared HMAC key.
///
/// Constructed with the same secret bytes as the
/// [`IssuerConfig`](crate::issuer::IssuerConfig) on the producing
/// side.  Production deployments would derive both from a shared
/// secret manager / KMS rather than env vars.
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    secret: Vec<u8>,
}

impl ValidatorConfig {
    /// Construct from a shared HMAC secret.
    #[must_use]
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    /// Borrow the secret.
    #[must_use]
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }
}

/// Validate `token` and project it into a [`Principal`].
///
/// Verifies signature and standard time claims (`exp`, `iat`).
///
/// # Errors
/// - [`Error::Jwt`] if the token signature is invalid or the token
///   is expired.
/// - [`Error::InvalidClaim`] if `sub` is not a `Uuid` or `iat`/`exp`
///   are outside chrono's representable range.
pub fn validate(config: &ValidatorConfig, token: &str) -> Result<Principal, Error> {
    let validation = Validation::new(Algorithm::HS256);
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret()),
        &validation,
    )?;
    claims_to_principal(&data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestError(String);

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl core::error::Error for TestError {}

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        if cond {
            Ok(())
        } else {
            Err(TestError(reason()))
        }
    }

    #[test]
    fn validate_rejects_garbage_token() -> Result<(), TestError> {
        let validator = ValidatorConfig::new(b"any-secret".to_vec());
        let outcome = validate(&validator, "not.a.token");
        check(matches!(outcome, Err(Error::Jwt { .. })), || {
            format!("got {outcome:?}")
        })
    }
}

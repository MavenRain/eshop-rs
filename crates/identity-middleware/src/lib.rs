//! axum extractor that validates a JWT bearer token and yields a
//! [`Principal`](identity_tokens::Principal).
//!
//! # Wiring
//!
//! 1. Construct a [`ValidatorState`] at `AppHost` startup from the
//!    same HMAC secret the Identity service signs with.
//! 2. Embed it in each context's `AppState` and implement
//!    [`axum::extract::FromRef<MyAppState> for ValidatorState`] so
//!    the extractor can pull it out per request.
//! 3. Add an [`AuthenticatedPrincipal`] argument to handlers that
//!    require authentication.  Missing or malformed tokens surface
//!    as `401 Unauthorized` automatically.

use std::sync::Arc;

use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use identity_tokens::{Principal, ValidatorConfig, validate};

const BEARER_PREFIX: &str = "Bearer ";

/// Shared validator state.  Cloneable; carries an [`Arc`] internally
/// so per-handler clones are cheap.
#[derive(Clone)]
pub struct ValidatorState {
    config: Arc<ValidatorConfig>,
}

impl ValidatorState {
    /// Construct from a [`ValidatorConfig`].
    #[must_use]
    pub fn new(config: ValidatorConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Borrow the validator config.
    #[must_use]
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }
}

/// axum extractor: extracts and validates a bearer token from the
/// `Authorization` header.  Consumers pull the inner
/// [`Principal`](identity_tokens::Principal) via
/// [`AuthenticatedPrincipal::into_inner`] /
/// [`AuthenticatedPrincipal::principal`].
///
/// # Rejections
/// Any of the following yield `401 Unauthorized`:
/// - missing `Authorization` header
/// - header value not `UTF-8`
/// - header value missing the `Bearer ` prefix
/// - token signature invalid or token expired
/// - claims malformed (`sub` not a `Uuid`, etc.)
pub struct AuthenticatedPrincipal(Principal);

impl AuthenticatedPrincipal {
    /// Borrow the inner principal.
    #[must_use]
    pub fn principal(&self) -> &Principal {
        &self.0
    }

    /// Move the inner principal out.
    #[must_use]
    pub fn into_inner(self) -> Principal {
        self.0
    }
}

impl<S> FromRequestParts<S> for AuthenticatedPrincipal
where
    ValidatorState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let validator = ValidatorState::from_ref(state);
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "missing Authorization header".to_string(),
                )
            })?;
        let value = header.to_str().map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Authorization header is not valid UTF-8".to_string(),
            )
        })?;
        let token = value.strip_prefix(BEARER_PREFIX).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                "expected Authorization: Bearer <token>".to_string(),
            )
        })?;
        let principal = validate(validator.config(), token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
        Ok(Self(principal))
    }
}

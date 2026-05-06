//! Shared application state for the webhooks HTTP API.

use std::sync::Arc;

use axum::extract::FromRef;
use identity_middleware::ValidatorState;
use toasty::Db;

/// State injected into every webhooks handler.
///
/// `Db` is shared via [`Arc`] for the same FFI reason as
/// `ordering-api::AppState`; `validator` is the JWT
/// [`ValidatorState`] the
/// [`AuthenticatedPrincipal`](identity_middleware::AuthenticatedPrincipal)
/// extractor uses.
#[derive(Clone)]
pub struct AppState {
    db: Arc<Db>,
    validator: ValidatorState,
}

impl AppState {
    /// Construct from an already-built [`Db`] and a JWT validator.
    #[must_use]
    pub fn new(db: Arc<Db>, validator: ValidatorState) -> Self {
        Self { db, validator }
    }

    /// Borrow the shared [`Db`] handle.
    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }
}

impl FromRef<AppState> for ValidatorState {
    fn from_ref(state: &AppState) -> Self {
        state.validator.clone()
    }
}

//! Shared application state.

use std::sync::Arc;

use axum::extract::FromRef;
use identity_middleware::ValidatorState;
use toasty::Db;

/// State injected into every handler.
///
/// `Db` is shared via [`Arc`] because [`Db::connection`] takes `&self`;
/// each handler pulls a per-request connection from the pool, then opens
/// a transaction on that connection.
///
/// `validator` is the JWT [`ValidatorState`] handlers use indirectly
/// through the
/// [`AuthenticatedPrincipal`](identity_middleware::AuthenticatedPrincipal)
/// extractor; the [`FromRef`] impl below makes it available to that
/// extractor.
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

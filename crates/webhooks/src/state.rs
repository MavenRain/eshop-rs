//! Shared application state for the webhooks HTTP API.

use std::sync::Arc;

use toasty::Db;

/// State injected into every webhooks handler.
///
/// `Db` is shared via [`Arc`] for the same FFI reason as
/// `ordering-api::AppState`.
#[derive(Clone)]
pub struct AppState {
    db: Arc<Db>,
}

impl AppState {
    /// Construct from an already-built [`Db`].
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Borrow the shared [`Db`] handle.
    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }
}

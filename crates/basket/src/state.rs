//! Shared application state for the basket HTTP API.

use std::sync::Arc;

use toasty::Db;

/// State injected into every basket handler.
///
/// `Db` is shared via [`Arc`] for the same FFI reason as
/// [`ordering_api::AppState`](https://docs.rs/ordering-api): toasty's
/// [`Db::connection`] takes `&self`, so callers pull a per-request
/// connection from the pool and open a transaction on it.
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

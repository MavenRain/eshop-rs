//! Shared application state for the catalog HTTP API.

use std::sync::Arc;

use toasty::Db;

/// State injected into every handler.
///
/// `Db` is shared via [`Arc`] because [`Db::connection`] takes
/// `&self`; each handler pulls a per-request connection from the
/// pool, then opens a transaction on that connection.
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

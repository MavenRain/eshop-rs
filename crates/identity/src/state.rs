//! Shared application state for the identity HTTP API.

use std::sync::Arc;

use identity_tokens::IssuerConfig;
use toasty::Db;

/// State injected into every identity handler.  Carries the shared
/// toasty [`Db`] and the JWT [`IssuerConfig`] (HMAC secret + TTL).
#[derive(Clone)]
pub struct AppState {
    db: Arc<Db>,
    issuer: Arc<IssuerConfig>,
}

impl AppState {
    /// Construct from an already-built [`Db`] and an
    /// [`IssuerConfig`].  Both are wrapped in [`Arc`] internally so
    /// the state cheaply clones into each handler.
    #[must_use]
    pub fn new(db: Arc<Db>, issuer: IssuerConfig) -> Self {
        Self {
            db,
            issuer: Arc::new(issuer),
        }
    }

    /// Borrow the shared [`Db`] handle.
    #[must_use]
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Borrow the JWT issuer configuration.
    #[must_use]
    pub fn issuer(&self) -> &IssuerConfig {
        &self.issuer
    }
}

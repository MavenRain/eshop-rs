//! Crate error type for the `AppHost` binary.
//!
//! Wraps every immediate dependency the binary touches: toasty, the
//! two API/processor crates, the event bus, axum / tokio
//! infrastructure failures, and config parsing.

/// Errors produced by the `AppHost`.
#[derive(Debug)]
pub enum Error {
    /// Configuration parsing failure (missing or malformed env var).
    Config { reason: String },
    /// Raw toasty driver failure.
    Toasty { reason: String },
    /// Ordering processor failure surfaced from `drain_once`.
    OrderingProcessor { reason: String },
    /// Catalog processor failure surfaced from `drain_once`.
    CatalogProcessor { reason: String },
    /// `event-bus-rabbitmq` connection or wiring failure.
    Bus { reason: String },
    /// `tokio::net::TcpListener::bind` failure.
    Bind { reason: String },
    /// `axum::serve` returned an I/O error.
    Serve { reason: String },
    /// `tokio::spawn` join failure (panic or cancellation).
    Join { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Config { reason } => write!(f, "config: {reason}"),
            Self::Toasty { reason } => write!(f, "toasty: {reason}"),
            Self::OrderingProcessor { reason } => write!(f, "ordering processor: {reason}"),
            Self::CatalogProcessor { reason } => write!(f, "catalog processor: {reason}"),
            Self::Bus { reason } => write!(f, "bus: {reason}"),
            Self::Bind { reason } => write!(f, "bind: {reason}"),
            Self::Serve { reason } => write!(f, "serve: {reason}"),
            Self::Join { reason } => write!(f, "join: {reason}"),
        }
    }
}

impl core::error::Error for Error {}

impl From<toasty::Error> for Error {
    fn from(e: toasty::Error) -> Self {
        Self::Toasty {
            reason: e.to_string(),
        }
    }
}

impl From<ordering_processor::Error> for Error {
    fn from(e: ordering_processor::Error) -> Self {
        Self::OrderingProcessor {
            reason: e.to_string(),
        }
    }
}

impl From<catalog_processor::Error> for Error {
    fn from(e: catalog_processor::Error) -> Self {
        Self::CatalogProcessor {
            reason: e.to_string(),
        }
    }
}

impl From<event_bus::Error> for Error {
    fn from(e: event_bus::Error) -> Self {
        Self::Bus {
            reason: e.to_string(),
        }
    }
}

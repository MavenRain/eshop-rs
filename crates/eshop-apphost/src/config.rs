//! Environment-driven configuration for the `AppHost`.
//!
//! Read at startup; nothing is re-read at runtime.  Each value comes
//! from a single environment variable with a documented default; the
//! defaults match the upstream `dotnet/eShop` `AppHost` defaults so
//! local dev parity is preserved.

use core::str::FromStr;
use core::time::Duration;
use std::env;

use event_bus_rabbitmq::{ExchangeName, QueueName, RmqConfig};

use crate::error::Error;

/// Resolved `AppHost` configuration.
pub struct Config {
    bind_address: String,
    amqp_uri: String,
    exchange: ExchangeName,
    ordering_queue: QueueName,
    catalog_queue: QueueName,
    poll_interval: Duration,
}

impl Config {
    /// Load from the process environment.
    ///
    /// # Errors
    /// [`Error::Config`] if any var is present but malformed.  Missing
    /// vars fall back to their documented defaults.
    pub fn from_env() -> Result<Self, Error> {
        let bind_address = read_string("ESHOP_BIND_ADDRESS", "127.0.0.1:8080");
        let amqp_uri = read_string("ESHOP_AMQP_URI", "amqp://guest:guest@localhost:5672/");
        let exchange = ExchangeName::from(read_string("ESHOP_EXCHANGE", "eshop_event_bus"));
        let ordering_queue =
            QueueName::from(read_string("ESHOP_ORDERING_QUEUE", "eshop-ordering-queue"));
        let catalog_queue =
            QueueName::from(read_string("ESHOP_CATALOG_QUEUE", "eshop-catalog-queue"));
        let poll_interval = read_duration_seconds("ESHOP_POLL_INTERVAL_SECONDS", 15)?;
        Ok(Self {
            bind_address,
            amqp_uri,
            exchange,
            ordering_queue,
            catalog_queue,
            poll_interval,
        })
    }

    /// Bind address for the combined HTTP server (e.g. `127.0.0.1:8080`).
    #[must_use]
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    /// Build the [`RmqConfig`] for the Ordering bus.
    #[must_use]
    pub fn ordering_rmq(&self) -> RmqConfig {
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            self.ordering_queue.clone(),
        )
    }

    /// Build the [`RmqConfig`] for the Catalog bus.
    #[must_use]
    pub fn catalog_rmq(&self) -> RmqConfig {
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            self.catalog_queue.clone(),
        )
    }

    /// Polling interval consulted by both processors.
    #[must_use]
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }
}

fn read_string(var: &str, default: &str) -> String {
    env::var(var).unwrap_or_else(|_| default.to_string())
}

fn read_duration_seconds(var: &str, default_secs: u64) -> Result<Duration, Error> {
    env::var(var)
        .ok()
        .map(|raw| {
            u64::from_str(&raw).map_err(|e| Error::Config {
                reason: format!("{var}: expected u64 seconds, got {raw:?}: {e}"),
            })
        })
        .transpose()
        .map(|opt| Duration::from_secs(opt.unwrap_or(default_secs)))
}

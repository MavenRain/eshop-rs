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
use identity_tokens::IssuerConfig;

use crate::error::Error;

/// Resolved `AppHost` configuration.
pub struct Config {
    bind_address: String,
    database_url: String,
    amqp_uri: String,
    exchange: ExchangeName,
    ordering_queue: QueueName,
    catalog_queue: QueueName,
    catalog_subscriber_queue: QueueName,
    basket_queue: QueueName,
    ordering_subscriber_queue: QueueName,
    webhooks_delivery_queue_prefix: String,
    jwt_secret: Vec<u8>,
    jwt_ttl: Duration,
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
        let database_url = read_string("ESHOP_DATABASE_URL", "sqlite::memory:");
        let amqp_uri = read_string("ESHOP_AMQP_URI", "amqp://guest:guest@localhost:5672/");
        let exchange = ExchangeName::from(read_string("ESHOP_EXCHANGE", "eshop_event_bus"));
        let ordering_queue =
            QueueName::from(read_string("ESHOP_ORDERING_QUEUE", "eshop-ordering-queue"));
        let catalog_queue =
            QueueName::from(read_string("ESHOP_CATALOG_QUEUE", "eshop-catalog-queue"));
        let catalog_subscriber_queue = QueueName::from(read_string(
            "ESHOP_CATALOG_SUBSCRIBER_QUEUE",
            "eshop-catalog-subscriber-queue",
        ));
        let basket_queue = QueueName::from(read_string("ESHOP_BASKET_QUEUE", "eshop-basket-queue"));
        let ordering_subscriber_queue = QueueName::from(read_string(
            "ESHOP_ORDERING_SUBSCRIBER_QUEUE",
            "eshop-ordering-subscriber-queue",
        ));
        let webhooks_delivery_queue_prefix = read_string(
            "ESHOP_WEBHOOKS_DELIVERY_QUEUE_PREFIX",
            "eshop-webhooks-delivery",
        );
        let jwt_secret =
            read_string("ESHOP_JWT_SECRET", "dev-only-jwt-secret-change-me").into_bytes();
        let jwt_ttl = read_duration_seconds("ESHOP_JWT_TTL_SECONDS", 3600)?;
        let poll_interval = read_duration_seconds("ESHOP_POLL_INTERVAL_SECONDS", 15)?;
        Ok(Self {
            bind_address,
            database_url,
            amqp_uri,
            exchange,
            ordering_queue,
            catalog_queue,
            catalog_subscriber_queue,
            basket_queue,
            ordering_subscriber_queue,
            webhooks_delivery_queue_prefix,
            jwt_secret,
            jwt_ttl,
            poll_interval,
        })
    }

    /// Bind address for the combined HTTP server (e.g. `127.0.0.1:8080`).
    #[must_use]
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    /// Toasty database URL.  Recognized schemes:
    /// - `sqlite::memory:` — ephemeral in-memory database (default).
    /// - `sqlite:<path>` — file-backed `SQLite` at the given path.
    /// - `postgresql://...` / `postgres://...` — `PostgreSQL`
    ///   connection URL passed straight through to
    ///   `toasty-driver-postgresql`.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
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

    /// Build the [`RmqConfig`] for the Catalog-side subscriber that
    /// consumes ordering integration events.  Distinct queue from the
    /// catalog publisher so the two consumer groups stay independent.
    #[must_use]
    pub fn catalog_subscriber_rmq(&self) -> RmqConfig {
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            self.catalog_subscriber_queue.clone(),
        )
    }

    /// Build the [`RmqConfig`] for the Basket bus.
    #[must_use]
    pub fn basket_rmq(&self) -> RmqConfig {
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            self.basket_queue.clone(),
        )
    }

    /// Build the [`RmqConfig`] for the Ordering-side subscriber that
    /// consumes basket integration events.  Distinct queue from the
    /// ordering publisher so the two consumer groups stay independent.
    #[must_use]
    pub fn ordering_subscriber_rmq(&self) -> RmqConfig {
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            self.ordering_subscriber_queue.clone(),
        )
    }

    /// Build the [`RmqConfig`] for the webhooks delivery worker's
    /// `<prefix>-<context>` queue, where `<context>` distinguishes the
    /// integration-event family being consumed (one queue per type).
    #[must_use]
    pub fn webhooks_delivery_rmq(&self, context: &str) -> RmqConfig {
        let queue = format!("{}-{}", self.webhooks_delivery_queue_prefix, context);
        RmqConfig::new(
            self.amqp_uri.clone(),
            self.exchange.clone(),
            QueueName::from(queue),
        )
    }

    /// Build the [`IssuerConfig`] for the Identity service.  Uses
    /// `chrono::Duration` internally because that is what the
    /// `identity-tokens` API accepts; converted from the `Duration`
    /// captured here.
    #[must_use]
    pub fn jwt_issuer(&self) -> IssuerConfig {
        let ttl_secs = i64::try_from(self.jwt_ttl.as_secs()).unwrap_or(i64::MAX);
        IssuerConfig::new(self.jwt_secret.clone(), chrono::Duration::seconds(ttl_secs))
    }

    /// Polling interval consulted by every processor.
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

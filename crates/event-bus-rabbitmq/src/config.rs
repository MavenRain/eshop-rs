//! `RabbitMQ` bus configuration.

/// Newtype for the AMQP exchange name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExchangeName(String);

impl ExchangeName {
    /// Default exchange used by upstream `dotnet/eShop`.
    #[must_use]
    pub fn eshop_default() -> Self {
        Self("eshop_event_bus".to_string())
    }

    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ExchangeName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ExchangeName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<ExchangeName> for String {
    fn from(e: ExchangeName) -> Self {
        e.0
    }
}

/// Newtype for the AMQP queue name a subscriber binds to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueName(String);

impl QueueName {
    /// Default queue name; appropriate when only one consumer of this binary
    /// is in flight.  Real deployments should pick a stable per-service name.
    #[must_use]
    pub fn eshop_default() -> Self {
        Self("eshop-default-queue".to_string())
    }

    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for QueueName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for QueueName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<QueueName> for String {
    fn from(q: QueueName) -> Self {
        q.0
    }
}

/// Construction parameters for [`crate::RabbitMqEventBus`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RmqConfig {
    amqp_uri: String,
    exchange: ExchangeName,
    queue_name: QueueName,
}

impl RmqConfig {
    /// Construct from a raw AMQP URI (e.g., `"amqp://guest:guest@localhost:5672/"`).
    #[must_use]
    pub fn new(amqp_uri: String, exchange: ExchangeName, queue_name: QueueName) -> Self {
        Self {
            amqp_uri,
            exchange,
            queue_name,
        }
    }

    /// AMQP URI.
    #[must_use]
    pub fn amqp_uri(&self) -> &str {
        &self.amqp_uri
    }

    /// Exchange name.
    #[must_use]
    pub fn exchange(&self) -> &ExchangeName {
        &self.exchange
    }

    /// Queue name (used by subscribers).
    #[must_use]
    pub fn queue_name(&self) -> &QueueName {
        &self.queue_name
    }
}

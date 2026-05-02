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

/// Construction parameters for [`crate::RabbitMqEventBus`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RmqConfig {
    amqp_uri: String,
    exchange: ExchangeName,
}

impl RmqConfig {
    /// Construct from a raw AMQP URI (e.g., `"amqp://guest:guest@localhost:5672/"`).
    #[must_use]
    pub fn new(amqp_uri: String, exchange: ExchangeName) -> Self {
        Self { amqp_uri, exchange }
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
}

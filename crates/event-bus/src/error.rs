//! Bus error type.

/// Errors produced by event bus operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Failed to serialize an event payload.
    Serialization { reason: String },
    /// Failed to deserialize an event payload.
    Deserialization { reason: String },
    /// Connection or channel-level failure with the broker.
    Connection { reason: String },
    /// Failure during publish.
    Publish { reason: String },
    /// Failure during subscribe / consume setup.
    Subscribe { reason: String },
    /// Inbound message named an event the application does not know about.
    UnknownEvent { name: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Serialization { reason } => write!(f, "serialization failed: {reason}"),
            Self::Deserialization { reason } => write!(f, "deserialization failed: {reason}"),
            Self::Connection { reason } => write!(f, "broker connection error: {reason}"),
            Self::Publish { reason } => write!(f, "publish failed: {reason}"),
            Self::Subscribe { reason } => write!(f, "subscribe failed: {reason}"),
            Self::UnknownEvent { name } => write!(f, "unknown event: {name}"),
        }
    }
}

impl std::error::Error for Error {}

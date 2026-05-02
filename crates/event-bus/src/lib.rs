//! Abstractions for an integration event bus.
//!
//! This crate is pure (no I/O, no async runtime).  It defines:
//!
//! - [`IntegrationEventMetadata`]: per-event id and creation timestamp.
//! - [`IntegrationEvent`]: trait every application event implements.
//! - [`EventBus`]: trait for any bus implementation, generic over the
//!   application's integration event sum type.
//! - [`Error`]: hand-rolled bus error enum.
//! - [`InMemoryEventBus`]: a synchronous in-memory implementation used by
//!   tests and downstream `#[cfg(test)]` integration tests.
//!
//! Concrete transports (`RabbitMQ`, NATS, etc.) live in sibling crates.

pub mod bus;
pub mod error;
pub mod event;
pub mod in_memory;
pub mod subscriber;

pub use bus::EventBus;
pub use error::Error;
pub use event::{IntegrationEvent, IntegrationEventMetadata};
pub use in_memory::InMemoryEventBus;
pub use subscriber::EventBusSubscriber;

//! [`EventBus`] trait.

use comp_cat_rs::effect::io::Io;

use crate::error::Error;
use crate::event::IntegrationEvent;

/// Trait for any event bus implementation.
///
/// Generic over the application's integration event type `E`.  Concrete
/// transports (`RabbitMQ`, NATS, etc.) implement this trait.
///
/// Operations return [`Io<Error, _>`](comp_cat_rs::effect::io::Io) so they
/// compose with the rest of the program inside `comp-cat-rs` effect chains.
pub trait EventBus<E: IntegrationEvent> {
    /// Publish a single event.
    fn publish(&self, event: E) -> Io<Error, ()>;
}

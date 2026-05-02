//! [`EventBusSubscriber`] trait.

use comp_cat_rs::effect::io::Io;

use crate::error::Error;
use crate::event::IntegrationEvent;

/// Trait for event bus subscriptions.
///
/// The handler is invoked once per inbound message, after deserialization.
/// Implementations are expected to set up the consumer infrastructure
/// (queues, bindings, consumer loop) when [`EventBusSubscriber::subscribe`]
/// is run, then drive the loop in the background until the bus instance is
/// dropped.  Handler errors are logged but do not stop the loop and do not
/// affect message acknowledgement (mirroring upstream `dotnet/eShop`
/// behavior; production deployments should add a dead-letter exchange).
///
/// `EventBus` and `EventBusSubscriber` are deliberately separate so test
/// implementations (e.g., [`InMemoryEventBus`](crate::InMemoryEventBus)) can
/// support publish-only without committing to a consumer infrastructure.
pub trait EventBusSubscriber<E: IntegrationEvent> {
    /// Subscribe to all of [`IntegrationEvent::all_event_names`] with
    /// `handler`.  The returned [`Io`] sets up the broker side and returns;
    /// the consumer loop continues running in the background.
    fn subscribe<F>(&self, handler: F) -> Io<Error, ()>
    where
        F: Fn(E) -> Io<Error, ()> + Send + Sync + 'static;
}

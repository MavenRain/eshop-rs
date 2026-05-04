//! Catalog-side handlers for integration events emitted by other
//! bounded contexts.
//!
//! Currently subscribes to a slim subset of
//! [`ordering_integration_events::OrderingIntegrationEvent`] —
//! [`OrderStatusChangedToAwaitingValidation`](crate::CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation)
//! and [`OrderStatusChangedToPaid`](crate::CatalogConsumedOrderingEvent::OrderStatusChangedToPaid)
//! —  with stub handler bodies that log and ack.
//!
//! See `handle`'s docs for the wire-validation scope: the handlers
//! today prove publish→subscribe round-trips through `RabbitMQ`; real
//! stock-reservation bodies land alongside the catalog/ordering
//! `ProductId` alignment slice.

mod consumed_event;
pub mod error;
mod handler;

pub use consumed_event::CatalogConsumedOrderingEvent;
pub use error::Error;
pub use handler::handle;

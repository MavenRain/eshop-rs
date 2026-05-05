//! Catalog-side handlers for integration events emitted by other
//! bounded contexts.
//!
//! Subscribes to a slim subset of
//! [`ordering_integration_events::OrderingIntegrationEvent`]:
//!
//! - [`OrderStatusChangedToAwaitingValidation`](crate::CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation):
//!   reserve stock by decrementing each
//!   [`CatalogItem`](catalog::CatalogItem)'s `available_stock` for the
//!   carried [`OrderStockItem`](ordering_integration_events::OrderStockItem)
//!   list, atomically inside a single toasty transaction.
//! - [`OrderStatusChangedToPaid`](crate::CatalogConsumedOrderingEvent::OrderStatusChangedToPaid):
//!   logged-only.  Upstream eShop commits the prior reservation here;
//!   in this port's reservation model, `AwaitingValidation` already
//!   decrements stock, so `Paid` is a no-op until a separate
//!   reservation/commit split lands.
//!
//! [`make_handler`] returns a `Fn(E) -> Io<event_bus::Error, ()>`
//! suitable for [`event_bus::EventBusSubscriber::subscribe`], capturing
//! the shared [`toasty::Db`] handle the `AppHost` passes in.

mod consumed_event;
pub mod error;
mod handler;

pub use consumed_event::CatalogConsumedOrderingEvent;
pub use error::Error;
pub use handler::make_handler;

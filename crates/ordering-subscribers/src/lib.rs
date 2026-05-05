//! Ordering-side handlers for integration events emitted by other
//! bounded contexts.
//!
//! Subscribes to a slim subset of
//! [`basket_integration_events::BasketIntegrationEvent`]:
//!
//! - [`UserCheckoutAccepted`](crate::OrderingConsumedBasketEvent::UserCheckoutAccepted):
//!   mints an [`Order`](ordering_domain::Order) from the carried
//!   basket snapshot.  Address and card fields fill from sentinels
//!   pending an identity bounded context; line items are real and
//!   carry the snapshot's product id, name, unit price, and units.
//!
//! [`make_handler`] returns a `Fn(E) -> Io<event_bus::Error, ()>`
//! suitable for [`event_bus::EventBusSubscriber::subscribe`], capturing
//! the shared [`toasty::Db`] handle the `AppHost` passes in.

mod consumed_event;
pub mod error;
mod handler;

pub use consumed_event::OrderingConsumedBasketEvent;
pub use error::Error;
pub use handler::make_handler;

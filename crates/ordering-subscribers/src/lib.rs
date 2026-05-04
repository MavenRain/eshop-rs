//! Ordering-side handlers for integration events emitted by other
//! bounded contexts.
//!
//! Currently subscribes to a slim subset of
//! [`basket_integration_events::BasketIntegrationEvent`] —
//! [`UserCheckoutAccepted`](crate::OrderingConsumedBasketEvent::UserCheckoutAccepted)
//! — with a stub handler body that logs and acks.
//!
//! Mirror of `catalog-subscribers`.  The handler today proves
//! basket→ordering publish/subscribe round-trips through `RabbitMQ`;
//! the real "mint an order from the basket snapshot" body lands
//! alongside the catalog/ordering `ProductId` alignment slice.

mod consumed_event;
pub mod error;
mod handler;

pub use consumed_event::OrderingConsumedBasketEvent;
pub use error::Error;
pub use handler::handle;

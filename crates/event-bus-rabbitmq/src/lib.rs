//! RabbitMQ-backed [`event_bus::EventBus`] implementation, on `lapin`.
//!
//! Wire compatible with upstream `dotnet/eShop`'s `RabbitMQEventBus`:
//!
//! - Direct exchange named `eshop_event_bus`.
//! - Routing key is the event name (per [`event_bus::IntegrationEvent::event_name`]).
//! - Body is `serde_json` of the event.
//! - Persistent delivery mode (delivery mode 2).
//!
//! Currently ships publish.  Subscribe (consumer loop, queue binding,
//! handler dispatch) is the next sub-step.
//!
//! ## Runtime requirement
//!
//! [`RabbitMqEventBus`] owns a dedicated [`tokio::runtime::Runtime`] and
//! drives broker operations on it.  The bus itself is sync from the outside
//! and must not be used from a thread that is already a tokio worker; use
//! it from the application's main thread or from a thread spawned outside
//! any tokio context.

pub mod bus;
pub mod config;

pub use bus::RabbitMqEventBus;
pub use config::{ExchangeName, RmqConfig};

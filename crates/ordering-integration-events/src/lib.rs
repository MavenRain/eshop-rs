//! Wire-form integration events for the Ordering bounded context.
//!
//! Domain events ([`ordering_domain::DomainEvent`]) live inside the
//! Ordering service.  When a domain event needs to cross a service
//! boundary, the API layer projects it into an
//! [`OrderingIntegrationEvent`] (this crate) and persists the JSON
//! representation in the outbox.  A separate publisher worker reads
//! the outbox and forwards the integration event onto the bus.
//!
//! The wire format mirrors upstream eShop's `Ordering.IntegrationEvents`
//! namespace: each variant tag is the C# class name (for example
//! `OrderStartedIntegrationEvent`) so consumers written against the
//! upstream contract decode the payload unchanged.
//!
//! # Examples
//!
//! ```
//! use ordering_integration_events::{
//!     OrderStartedIntegrationEventPayload, OrderingIntegrationEvent,
//! };
//! use uuid::Uuid;
//!
//! let event = OrderingIntegrationEvent::OrderStarted(
//!     OrderStartedIntegrationEventPayload::new(
//!         Uuid::nil(),
//!         "user-123".to_string(),
//!         "Alice".to_string(),
//!     ),
//! );
//! let bytes = serde_json::to_vec(&event)?;
//! let _parsed: OrderingIntegrationEvent = serde_json::from_slice(&bytes)?;
//! # Ok::<(), serde_json::Error>(())
//! ```

mod event;
mod from_domain;
mod payload;
mod stock_item;

pub use event::{
    ORDER_CANCELLED, ORDER_SHIPPED, ORDER_STARTED, ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION,
    ORDER_STATUS_CHANGED_TO_PAID, ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
    OrderingIntegrationEvent,
};
pub use from_domain::from_domain_event;
pub use payload::{
    OrderCancelledIntegrationEventPayload, OrderShippedIntegrationEventPayload,
    OrderStartedIntegrationEventPayload,
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload,
    OrderStatusChangedToStockConfirmedIntegrationEventPayload,
};
pub use stock_item::OrderStockItem;

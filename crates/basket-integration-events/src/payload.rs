//! Payload struct for the single
//! [`BasketIntegrationEvent::UserCheckoutAccepted`](crate::BasketIntegrationEvent::UserCheckoutAccepted)
//! variant.
//!
//! Mirrors upstream eShop's `UserCheckoutAcceptedIntegrationEvent`,
//! trimmed to the fields our Rust port currently surfaces.  Upstream
//! also carries shipping address, card details, and a full
//! `CustomerBasket` snapshot; those land alongside an identity
//! bounded context, which the port has not added yet.  Reused
//! [`OrderStockItem`] from `ordering-integration-events` so the
//! basket→ordering hand-off speaks the same primitive shape.

use ordering_integration_events::OrderStockItem;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload of [`BasketIntegrationEvent::UserCheckoutAccepted`](crate::BasketIntegrationEvent::UserCheckoutAccepted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCheckoutAcceptedIntegrationEventPayload {
    customer_id: Uuid,
    request_id: Uuid,
    order_stock_items: Vec<OrderStockItem>,
}

impl UserCheckoutAcceptedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(
        customer_id: Uuid,
        request_id: Uuid,
        order_stock_items: Vec<OrderStockItem>,
    ) -> Self {
        Self {
            customer_id,
            request_id,
            order_stock_items,
        }
    }

    /// Customer who initiated the checkout.  Becomes the buyer id
    /// on the ordering side.
    #[must_use]
    pub fn customer_id(&self) -> Uuid {
        self.customer_id
    }

    /// Idempotency key.  Ordering uses this to dedupe retried checkouts.
    #[must_use]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Snapshot of the basket's items at checkout time.
    #[must_use]
    pub fn order_stock_items(&self) -> &[OrderStockItem] {
        &self.order_stock_items
    }
}

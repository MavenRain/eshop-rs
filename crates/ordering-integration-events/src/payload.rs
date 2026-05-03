//! Payload structs for each [`OrderingIntegrationEvent`](crate::OrderingIntegrationEvent)
//! variant.
//!
//! Fields are private; serde derives access them through the
//! `#[derive(Serialize, Deserialize)]` macro expansion, so the wire
//! format uses the field names directly.  External callers go through
//! the explicit constructors and getters.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::stock_item::OrderStockItem;

/// Payload of [`OrderingIntegrationEvent::OrderStarted`](crate::OrderingIntegrationEvent::OrderStarted).
///
/// Card details are deliberately omitted: they live inside the Ordering
/// service and never cross the bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStartedIntegrationEventPayload {
    order_id: Uuid,
    user_id: String,
    user_name: String,
}

impl OrderStartedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid, user_id: String, user_name: String) -> Self {
        Self {
            order_id,
            user_id,
            user_name,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }

    /// User identifier.
    #[must_use]
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// User display name.
    #[must_use]
    pub fn user_name(&self) -> &str {
        &self.user_name
    }
}

/// Payload of
/// [`OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation`](crate::OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStatusChangedToAwaitingValidationIntegrationEventPayload {
    order_id: Uuid,
    order_stock_items: Vec<OrderStockItem>,
}

impl OrderStatusChangedToAwaitingValidationIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid, order_stock_items: Vec<OrderStockItem>) -> Self {
        Self {
            order_id,
            order_stock_items,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }

    /// Snapshot of the order's line items.
    #[must_use]
    pub fn order_stock_items(&self) -> &[OrderStockItem] {
        &self.order_stock_items
    }
}

/// Payload of
/// [`OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed`](crate::OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStatusChangedToStockConfirmedIntegrationEventPayload {
    order_id: Uuid,
}

impl OrderStatusChangedToStockConfirmedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }
}

/// Payload of
/// [`OrderingIntegrationEvent::OrderStatusChangedToPaid`](crate::OrderingIntegrationEvent::OrderStatusChangedToPaid).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStatusChangedToPaidIntegrationEventPayload {
    order_id: Uuid,
    order_stock_items: Vec<OrderStockItem>,
}

impl OrderStatusChangedToPaidIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid, order_stock_items: Vec<OrderStockItem>) -> Self {
        Self {
            order_id,
            order_stock_items,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }

    /// Snapshot of the order's line items.
    #[must_use]
    pub fn order_stock_items(&self) -> &[OrderStockItem] {
        &self.order_stock_items
    }
}

/// Payload of [`OrderingIntegrationEvent::OrderShipped`](crate::OrderingIntegrationEvent::OrderShipped).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderShippedIntegrationEventPayload {
    order_id: Uuid,
}

impl OrderShippedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }
}

/// Payload of [`OrderingIntegrationEvent::OrderCancelled`](crate::OrderingIntegrationEvent::OrderCancelled).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCancelledIntegrationEventPayload {
    order_id: Uuid,
}

impl OrderCancelledIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(order_id: Uuid) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> Uuid {
        self.order_id
    }
}

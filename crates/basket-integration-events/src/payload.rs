//! Payload struct for the single
//! [`BasketIntegrationEvent::UserCheckoutAccepted`](crate::BasketIntegrationEvent::UserCheckoutAccepted)
//! variant, plus the [`BasketSnapshotItem`] line-item shape carried
//! inside it.
//!
//! Mirrors upstream eShop's `UserCheckoutAcceptedIntegrationEvent`,
//! trimmed to the fields our Rust port currently surfaces.  Upstream
//! also carries shipping address and card details that depend on an
//! identity bounded context the port has not added yet; until then
//! the ordering side fills those with sentinel values when minting
//! an order from the carried basket snapshot.
//!
//! `BasketSnapshotItem` carries the full denormalized line item
//! (product id, product name, unit price, picture url, units) so the
//! ordering subscriber can construct an `OrderItem` without doing a
//! cross-context lookup against catalog.  Distinct from
//! `OrderStockItem` (which is the slim `(product_id, units)`
//! primitive used by ordering→catalog stock-decrement events).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Denormalized snapshot of one basket line item, sufficient to
/// construct an [`OrderItem`](https://docs.rs/ordering-domain) on
/// the consuming side without a cross-context lookup against
/// catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasketSnapshotItem {
    product_id: i32,
    product_name: String,
    unit_price: Decimal,
    picture_url: Option<String>,
    units: u32,
}

impl BasketSnapshotItem {
    /// Construct.
    #[must_use]
    pub fn new(
        product_id: i32,
        product_name: String,
        unit_price: Decimal,
        picture_url: Option<String>,
        units: u32,
    ) -> Self {
        Self {
            product_id,
            product_name,
            unit_price,
            picture_url,
            units,
        }
    }

    /// Catalog product id.
    #[must_use]
    pub fn product_id(&self) -> i32 {
        self.product_id
    }

    /// Snapshotted product name.
    #[must_use]
    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    /// Snapshotted unit price.
    #[must_use]
    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    /// Snapshotted picture url, if the basket item carried one.
    #[must_use]
    pub fn picture_url(&self) -> Option<&str> {
        self.picture_url.as_deref()
    }

    /// Quantity ordered.
    #[must_use]
    pub fn units(&self) -> u32 {
        self.units
    }
}

/// Payload of [`BasketIntegrationEvent::UserCheckoutAccepted`](crate::BasketIntegrationEvent::UserCheckoutAccepted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCheckoutAcceptedIntegrationEventPayload {
    customer_id: Uuid,
    request_id: Uuid,
    basket_items: Vec<BasketSnapshotItem>,
}

impl UserCheckoutAcceptedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(customer_id: Uuid, request_id: Uuid, basket_items: Vec<BasketSnapshotItem>) -> Self {
        Self {
            customer_id,
            request_id,
            basket_items,
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
    pub fn basket_items(&self) -> &[BasketSnapshotItem] {
        &self.basket_items
    }
}

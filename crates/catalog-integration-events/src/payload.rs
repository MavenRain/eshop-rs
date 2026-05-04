//! Payload struct for the single
//! [`CatalogIntegrationEvent::ProductPriceChanged`](crate::CatalogIntegrationEvent::ProductPriceChanged)
//! variant.
//!
//! Mirrors upstream eShop's `ProductPriceChangedIntegrationEvent` :
//! `(ProductId, NewPrice, OldPrice)`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload of [`CatalogIntegrationEvent::ProductPriceChanged`](crate::CatalogIntegrationEvent::ProductPriceChanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductPriceChangedIntegrationEventPayload {
    product_id: Uuid,
    new_price: Decimal,
    old_price: Decimal,
}

impl ProductPriceChangedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(product_id: Uuid, new_price: Decimal, old_price: Decimal) -> Self {
        Self {
            product_id,
            new_price,
            old_price,
        }
    }

    /// Product identifier.
    #[must_use]
    pub fn product_id(&self) -> Uuid {
        self.product_id
    }

    /// Price after the change.
    #[must_use]
    pub fn new_price(&self) -> Decimal {
        self.new_price
    }

    /// Price prior to the change.
    #[must_use]
    pub fn old_price(&self) -> Decimal {
        self.old_price
    }
}

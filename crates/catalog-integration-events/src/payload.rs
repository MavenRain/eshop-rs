//! Payload struct for the single
//! [`CatalogIntegrationEvent::ProductPriceChanged`](crate::CatalogIntegrationEvent::ProductPriceChanged)
//! variant.
//!
//! Mirrors upstream eShop's `ProductPriceChangedIntegrationEvent` :
//! `(ProductId, NewPrice, OldPrice)`.  `product_id` is `i32`,
//! aligning with [`catalog::CatalogItemId`](https://docs.rs/catalog),
//! [`ordering_domain::ProductId`](https://docs.rs/ordering-domain),
//! [`basket::ProductId`](https://docs.rs/basket), and the
//! [`OrderStockItem.product_id`](ordering_integration_events::OrderStockItem)
//! field, so a downstream consumer that joins price changes against
//! a stock-decrement flow speaks one identifier shape end to end.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Payload of [`CatalogIntegrationEvent::ProductPriceChanged`](crate::CatalogIntegrationEvent::ProductPriceChanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductPriceChangedIntegrationEventPayload {
    product_id: i32,
    new_price: Decimal,
    old_price: Decimal,
}

impl ProductPriceChangedIntegrationEventPayload {
    /// Construct.
    #[must_use]
    pub fn new(product_id: i32, new_price: Decimal, old_price: Decimal) -> Self {
        Self {
            product_id,
            new_price,
            old_price,
        }
    }

    /// Product identifier.
    #[must_use]
    pub fn product_id(&self) -> i32 {
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

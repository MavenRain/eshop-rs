//! Domain events emitted by the Catalog bounded context.
//!
//! Mirrors `eShop.Catalog.API.IntegrationEvents.Events` for v1.  The
//! single event upstream publishes from inside the bounded context is
//! [`ProductPriceChangedEvent`]; upstream additionally consumes ordering
//! events, but those are subscriptions, not emissions, and live in the
//! API layer.

use crate::item::CatalogItemId;
use crate::money::Price;

/// Domain events emitted by Catalog aggregates.
///
/// Sum-typed for forward compatibility: more variants will appear as
/// the catalog gains stock-confirmation, restock, and discontinuation
/// flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// A [`CatalogItem`](crate::item::CatalogItem)'s price changed.
    ProductPriceChanged(ProductPriceChangedEvent),
}

/// Payload of [`DomainEvent::ProductPriceChanged`].
///
/// Upstream's `ProductPriceChangedIntegrationEvent` is `(ProductId,
/// NewPrice, OldPrice)`; we keep the same triple, with [`Price`] in
/// place of bare `decimal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductPriceChangedEvent {
    product_id: CatalogItemId,
    new_price: Price,
    old_price: Price,
}

impl ProductPriceChangedEvent {
    /// Construct.
    #[must_use]
    pub fn new(product_id: CatalogItemId, new_price: Price, old_price: Price) -> Self {
        Self {
            product_id,
            new_price,
            old_price,
        }
    }

    /// Product identifier.
    #[must_use]
    pub fn product_id(&self) -> CatalogItemId {
        self.product_id
    }

    /// Price as of the change.
    #[must_use]
    pub fn new_price(&self) -> Price {
        self.new_price
    }

    /// Price prior to the change.
    #[must_use]
    pub fn old_price(&self) -> Price {
        self.old_price
    }
}

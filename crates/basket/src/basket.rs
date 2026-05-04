//! [`CustomerBasket`] aggregate.
//!
//! Models upstream eShop's `CustomerBasket`: a customer-keyed
//! collection of [`BasketItem`]s that the API replaces wholesale
//! on every update.  Has no domain events for v1; the
//! `UserCheckoutAcceptedIntegrationEvent` upstream emits at checkout
//! lands alongside the basket integration-events crate as a
//! follow-up slice.

use crate::basket_item::BasketItem;
use crate::customer::CustomerId;

/// A customer's current basket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerBasket {
    customer_id: CustomerId,
    items: Vec<BasketItem>,
}

impl CustomerBasket {
    /// Construct from a customer id and a (possibly empty) item list.
    #[must_use]
    pub fn new(customer_id: CustomerId, items: Vec<BasketItem>) -> Self {
        Self { customer_id, items }
    }

    /// Owning customer.
    #[must_use]
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    /// Borrow the line items.
    #[must_use]
    pub fn items(&self) -> &[BasketItem] {
        &self.items
    }

    /// Whether the basket has any items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Number of distinct line items.
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

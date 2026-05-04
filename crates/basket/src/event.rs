//! Domain events emitted by the Basket bounded context.
//!
//! Currently a single variant fired by the checkout endpoint.  More
//! variants will appear as the basket grows abandonment / reminder
//! flows.

use crate::basket_item::BasketItem;
use crate::customer::CustomerId;
use uuid::Uuid;

/// Domain events emitted by Basket aggregates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// A customer accepted their checkout.  Ordering will mint an
    /// order from the carried snapshot.
    CheckoutAccepted(CheckoutAcceptedEvent),
}

/// Payload of [`DomainEvent::CheckoutAccepted`].
///
/// Carries the customer id, an idempotency key (`request_id`), and a
/// snapshot of the basket items as they were at checkout.  Mirrors
/// the upstream `UserCheckoutAcceptedIntegrationEvent` payload but
/// stays inside the domain newtypes; translation to wire types
/// happens in [`crate::outbox::from_domain_event`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutAcceptedEvent {
    customer_id: CustomerId,
    request_id: Uuid,
    items: Vec<BasketItem>,
}

impl CheckoutAcceptedEvent {
    /// Construct.
    #[must_use]
    pub fn new(customer_id: CustomerId, request_id: Uuid, items: Vec<BasketItem>) -> Self {
        Self {
            customer_id,
            request_id,
            items,
        }
    }

    /// Customer who initiated the checkout.
    #[must_use]
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    /// Idempotency key.
    #[must_use]
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    /// Snapshot of the basket items.
    #[must_use]
    pub fn items(&self) -> &[BasketItem] {
        &self.items
    }
}

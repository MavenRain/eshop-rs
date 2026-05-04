//! Slim view of [`BasketIntegrationEvent`](basket_integration_events::BasketIntegrationEvent):
//! only the variants the ordering bounded context consumes.
//!
//! Mirror of `catalog-subscribers::CatalogConsumedOrderingEvent`.
//! The basket service today publishes a single integration event
//! (`UserCheckoutAcceptedIntegrationEvent`), so this enum has the
//! same single variant — but kept as a slim, ordering-owned type so
//! the queue binding lists only the routing keys ordering cares
//! about, and so future basket variants don't auto-fan out to
//! ordering's consume loop.
//!
//! The serde tags match upstream eShop's class names exactly, so a
//! payload published by `basket-processor` deserializes into this
//! enum unchanged.

use basket_integration_events::{
    USER_CHECKOUT_ACCEPTED, UserCheckoutAcceptedIntegrationEventPayload,
};
use event_bus::IntegrationEvent;
use serde::{Deserialize, Serialize};

const ALL_EVENT_NAMES: &[&str] = &[USER_CHECKOUT_ACCEPTED];

/// Subset of [`BasketIntegrationEvent`](basket_integration_events::BasketIntegrationEvent)
/// the ordering context subscribes to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum OrderingConsumedBasketEvent {
    /// `UserCheckoutAcceptedIntegrationEvent`.  Ordering mints an
    /// order from the carried basket snapshot.
    #[serde(rename = "UserCheckoutAcceptedIntegrationEvent")]
    UserCheckoutAccepted(UserCheckoutAcceptedIntegrationEventPayload),
}

impl IntegrationEvent for OrderingConsumedBasketEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::UserCheckoutAccepted(_) => USER_CHECKOUT_ACCEPTED,
        }
    }

    fn all_event_names() -> &'static [&'static str] {
        ALL_EVENT_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use basket_integration_events::BasketIntegrationEvent;
    use uuid::Uuid;

    #[derive(Debug)]
    enum TestError {
        Json(String),
        Mismatch(String),
    }

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Json(reason) => write!(f, "json: {reason}"),
                Self::Mismatch(reason) => write!(f, "mismatch: {reason}"),
            }
        }
    }

    impl core::error::Error for TestError {}

    impl From<serde_json::Error> for TestError {
        fn from(e: serde_json::Error) -> Self {
            Self::Json(e.to_string())
        }
    }

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        if cond {
            Ok(())
        } else {
            Err(TestError::Mismatch(reason()))
        }
    }

    #[test]
    fn user_checkout_accepted_round_trips() -> Result<(), TestError> {
        let event = OrderingConsumedBasketEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(Uuid::nil(), Uuid::max(), Vec::new()),
        );
        let bytes = serde_json::to_vec(&event)?;
        let parsed: OrderingConsumedBasketEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    /// Cross-format compat: a payload serialized as
    /// [`BasketIntegrationEvent::UserCheckoutAccepted`]
    /// must deserialize as the matching
    /// [`OrderingConsumedBasketEvent::UserCheckoutAccepted`] variant.
    /// This is the contract the basket publisher / ordering
    /// subscriber pair relies on.
    #[test]
    fn basket_publisher_payload_deserializes_into_consumed_subset() -> Result<(), TestError> {
        let customer_id = Uuid::nil();
        let request_id = Uuid::max();
        let publisher_view = BasketIntegrationEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(customer_id, request_id, Vec::new()),
        );
        let wire_bytes = serde_json::to_vec(&publisher_view)?;
        let consumer_view: OrderingConsumedBasketEvent = serde_json::from_slice(&wire_bytes)?;
        let payload_customer_id = match &consumer_view {
            OrderingConsumedBasketEvent::UserCheckoutAccepted(p) => p.customer_id(),
        };
        check(payload_customer_id == customer_id, || {
            format!("expected {customer_id}, got {payload_customer_id}")
        })
    }

    #[test]
    fn all_event_names_lists_only_user_checkout_accepted() -> Result<(), TestError> {
        let names = OrderingConsumedBasketEvent::all_event_names();
        check(names == [USER_CHECKOUT_ACCEPTED], || {
            format!("got {names:?}")
        })
    }

    #[test]
    fn event_name_matches_constant() -> Result<(), TestError> {
        let event = OrderingConsumedBasketEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(Uuid::nil(), Uuid::nil(), Vec::new()),
        );
        check(event.event_name() == USER_CHECKOUT_ACCEPTED, || {
            format!("got {}", event.event_name())
        })
    }
}

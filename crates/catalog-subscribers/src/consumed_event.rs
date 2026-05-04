//! Slim view of the [`OrderingIntegrationEvent`](ordering_integration_events::OrderingIntegrationEvent)
//! enum: only the variants the catalog actually consumes.
//!
//! Why a separate type instead of subscribing to the full
//! `OrderingIntegrationEvent`?
//!
//! - [`EventBusSubscriber::subscribe`](event_bus::EventBusSubscriber::subscribe)
//!   binds the queue to every routing key returned by
//!   `E::all_event_names()`.  Subscribing as the full enum would bind
//!   four extra routing keys (`OrderStarted`,
//!   `OrderStatusChangedToStockConfirmed`, `OrderShipped`,
//!   `OrderCancelled`) just to ack-and-drop them.  The slim enum binds
//!   only the two events the catalog cares about.
//! - The serde tags match upstream eShop's class names exactly, so a
//!   payload published by `ordering-processor` deserializes into this
//!   enum unchanged.

use event_bus::IntegrationEvent;
use ordering_integration_events::{
    ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION, ORDER_STATUS_CHANGED_TO_PAID,
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload,
};
use serde::{Deserialize, Serialize};

const ALL_EVENT_NAMES: &[&str] = &[
    ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION,
    ORDER_STATUS_CHANGED_TO_PAID,
];

/// Subset of [`OrderingIntegrationEvent`](ordering_integration_events::OrderingIntegrationEvent)
/// the catalog subscribes to.
///
/// The serde-tagged variant names match the upstream eShop class
/// names (and the corresponding `ORDER_STATUS_CHANGED_TO_*` constants
/// in `ordering-integration-events`), so a payload produced by the
/// ordering publisher deserializes into this enum unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum CatalogConsumedOrderingEvent {
    /// `OrderStatusChangedToAwaitingValidationIntegrationEvent`.
    /// Catalog reserves stock for the order's items.
    #[serde(rename = "OrderStatusChangedToAwaitingValidationIntegrationEvent")]
    OrderStatusChangedToAwaitingValidation(
        OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    ),
    /// `OrderStatusChangedToPaidIntegrationEvent`.  Catalog commits
    /// the previously-reserved decrement.
    #[serde(rename = "OrderStatusChangedToPaidIntegrationEvent")]
    OrderStatusChangedToPaid(OrderStatusChangedToPaidIntegrationEventPayload),
}

impl IntegrationEvent for CatalogConsumedOrderingEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::OrderStatusChangedToAwaitingValidation(_) => {
                ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION
            }
            Self::OrderStatusChangedToPaid(_) => ORDER_STATUS_CHANGED_TO_PAID,
        }
    }

    fn all_event_names() -> &'static [&'static str] {
        ALL_EVENT_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ordering_integration_events::{OrderStockItem, OrderingIntegrationEvent};
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
        match () {
            () if cond => Ok(()),
            () => Err(TestError::Mismatch(reason())),
        }
    }

    #[test]
    fn awaiting_validation_round_trips() -> Result<(), TestError> {
        let event = CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation(
            OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                Uuid::nil(),
                vec![OrderStockItem::new(42, 3)],
            ),
        );
        let bytes = serde_json::to_vec(&event)?;
        let parsed: CatalogConsumedOrderingEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    #[test]
    fn paid_round_trips() -> Result<(), TestError> {
        let event = CatalogConsumedOrderingEvent::OrderStatusChangedToPaid(
            OrderStatusChangedToPaidIntegrationEventPayload::new(
                Uuid::nil(),
                vec![OrderStockItem::new(7, 2)],
            ),
        );
        let bytes = serde_json::to_vec(&event)?;
        let parsed: CatalogConsumedOrderingEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    /// Cross-format compat: a payload serialized as
    /// [`OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation`]
    /// must deserialize as the matching
    /// [`CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation`]
    /// variant.  This is the contract the publisher / subscriber pair
    /// relies on.
    #[test]
    fn ordering_publisher_payload_deserializes_into_consumed_subset() -> Result<(), TestError> {
        let order_id = Uuid::nil();
        let items = vec![OrderStockItem::new(42, 3)];
        let publisher_view = OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(
            OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                order_id,
                items.clone(),
            ),
        );
        let wire_bytes = serde_json::to_vec(&publisher_view)?;
        let consumer_view: CatalogConsumedOrderingEvent = serde_json::from_slice(&wire_bytes)?;
        let payload_order_id = match &consumer_view {
            CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation(p) => p.order_id(),
            CatalogConsumedOrderingEvent::OrderStatusChangedToPaid(_) => Uuid::max(),
        };
        check(payload_order_id == order_id, || {
            format!("expected {order_id}, got {payload_order_id}")
        })
    }

    #[test]
    fn unsubscribed_variant_payload_does_not_deserialize() -> Result<(), TestError> {
        // `OrderStarted` is part of `OrderingIntegrationEvent` but not
        // `CatalogConsumedOrderingEvent`.  The serde-tagged enum must
        // reject the wire tag rather than silently mis-decode.
        let publisher_view = OrderingIntegrationEvent::OrderStarted(
            ordering_integration_events::OrderStartedIntegrationEventPayload::new(
                Uuid::nil(),
                "user".to_string(),
                "name".to_string(),
            ),
        );
        let wire_bytes = serde_json::to_vec(&publisher_view)?;
        let outcome: Result<CatalogConsumedOrderingEvent, _> = serde_json::from_slice(&wire_bytes);
        check(outcome.is_err(), || format!("got {outcome:?}"))
    }

    #[test]
    fn all_event_names_lists_the_two_subscribed_routes() -> Result<(), TestError> {
        let names = CatalogConsumedOrderingEvent::all_event_names();
        check(names.len() == 2, || format!("len {}", names.len()))?;
        check(
            names.contains(&ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION),
            || "missing AwaitingValidation".to_string(),
        )?;
        check(names.contains(&ORDER_STATUS_CHANGED_TO_PAID), || {
            "missing Paid".to_string()
        })
    }
}

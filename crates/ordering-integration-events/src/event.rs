//! [`OrderingIntegrationEvent`] sum type and its
//! [`IntegrationEvent`] implementation.
//!
//! The wire format uses an internally-tagged enum: every variant
//! serializes to a JSON object with an `event_name` discriminator and
//! the payload's fields flattened alongside.  Variant tags are
//! renamed to match upstream eShop's class names (e.g.
//! `OrderStartedIntegrationEvent`) so a downstream service written
//! against eShop's `IntegrationEvents` namespace can deserialize the
//! payload unchanged.

use event_bus::IntegrationEvent;
use serde::{Deserialize, Serialize};

use crate::payload::{
    OrderCancelledIntegrationEventPayload, OrderShippedIntegrationEventPayload,
    OrderStartedIntegrationEventPayload,
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload,
    OrderStatusChangedToStockConfirmedIntegrationEventPayload,
};

/// Stable wire name for [`OrderingIntegrationEvent::OrderStarted`].
pub const ORDER_STARTED: &str = "OrderStartedIntegrationEvent";
/// Stable wire name for [`OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation`].
pub const ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION: &str =
    "OrderStatusChangedToAwaitingValidationIntegrationEvent";
/// Stable wire name for [`OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed`].
pub const ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED: &str =
    "OrderStatusChangedToStockConfirmedIntegrationEvent";
/// Stable wire name for [`OrderingIntegrationEvent::OrderStatusChangedToPaid`].
pub const ORDER_STATUS_CHANGED_TO_PAID: &str = "OrderStatusChangedToPaidIntegrationEvent";
/// Stable wire name for [`OrderingIntegrationEvent::OrderShipped`].
pub const ORDER_SHIPPED: &str = "OrderShippedIntegrationEvent";
/// Stable wire name for [`OrderingIntegrationEvent::OrderCancelled`].
pub const ORDER_CANCELLED: &str = "OrderCancelledIntegrationEvent";

const ALL_EVENT_NAMES: &[&str] = &[
    ORDER_STARTED,
    ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION,
    ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
    ORDER_STATUS_CHANGED_TO_PAID,
    ORDER_SHIPPED,
    ORDER_CANCELLED,
];

/// Integration events emitted by the Ordering bounded context.
///
/// Note that [`ordering_domain::DomainEvent::BuyerPaymentMethodVerified`]
/// has no integration event: it is a pure in-process domain event,
/// matching upstream eShop's design.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum OrderingIntegrationEvent {
    /// An order has been created and submitted.
    #[serde(rename = "OrderStartedIntegrationEvent")]
    OrderStarted(OrderStartedIntegrationEventPayload),
    /// Status moved to `AwaitingValidation`.
    #[serde(rename = "OrderStatusChangedToAwaitingValidationIntegrationEvent")]
    OrderStatusChangedToAwaitingValidation(
        OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    ),
    /// Status moved to `StockConfirmed`.
    #[serde(rename = "OrderStatusChangedToStockConfirmedIntegrationEvent")]
    OrderStatusChangedToStockConfirmed(OrderStatusChangedToStockConfirmedIntegrationEventPayload),
    /// Status moved to `Paid`.
    #[serde(rename = "OrderStatusChangedToPaidIntegrationEvent")]
    OrderStatusChangedToPaid(OrderStatusChangedToPaidIntegrationEventPayload),
    /// Status moved to `Shipped`.
    #[serde(rename = "OrderShippedIntegrationEvent")]
    OrderShipped(OrderShippedIntegrationEventPayload),
    /// Status moved to `Cancelled`.
    #[serde(rename = "OrderCancelledIntegrationEvent")]
    OrderCancelled(OrderCancelledIntegrationEventPayload),
}

impl IntegrationEvent for OrderingIntegrationEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::OrderStarted(_) => ORDER_STARTED,
            Self::OrderStatusChangedToAwaitingValidation(_) => {
                ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION
            }
            Self::OrderStatusChangedToStockConfirmed(_) => ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
            Self::OrderStatusChangedToPaid(_) => ORDER_STATUS_CHANGED_TO_PAID,
            Self::OrderShipped(_) => ORDER_SHIPPED,
            Self::OrderCancelled(_) => ORDER_CANCELLED,
        }
    }

    fn all_event_names() -> &'static [&'static str] {
        ALL_EVENT_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    use crate::stock_item::OrderStockItem;

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
    fn order_started_round_trips() -> Result<(), TestError> {
        let event =
            OrderingIntegrationEvent::OrderStarted(OrderStartedIntegrationEventPayload::new(
                Uuid::nil(),
                "user-123".to_string(),
                "Alice".to_string(),
            ));
        let bytes = serde_json::to_vec(&event)?;
        let parsed: OrderingIntegrationEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    #[test]
    fn order_started_wire_tag() -> Result<(), TestError> {
        let event = OrderingIntegrationEvent::OrderStarted(
            OrderStartedIntegrationEventPayload::new(Uuid::nil(), "u".to_string(), "n".to_string()),
        );
        let value: serde_json::Value = serde_json::to_value(&event)?;
        let tag = value
            .get("event_name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .unwrap_or_default();
        check(tag == ORDER_STARTED, || format!("got tag {tag}"))
    }

    #[test]
    fn awaiting_validation_payload_carries_items() -> Result<(), TestError> {
        let event = OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(
            OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                Uuid::nil(),
                vec![OrderStockItem::new(42, 3)],
            ),
        );
        let bytes = serde_json::to_vec(&event)?;
        let parsed: OrderingIntegrationEvent = serde_json::from_slice(&bytes)?;
        check(parsed == event, || format!("got {parsed:?}"))
    }

    #[test]
    fn event_name_matches_wire_tag() -> Result<(), TestError> {
        let cases: [(OrderingIntegrationEvent, &'static str); 6] = [
            (
                OrderingIntegrationEvent::OrderStarted(OrderStartedIntegrationEventPayload::new(
                    Uuid::nil(),
                    String::new(),
                    String::new(),
                )),
                ORDER_STARTED,
            ),
            (
                OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(
                    OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                        Uuid::nil(),
                        Vec::new(),
                    ),
                ),
                ORDER_STATUS_CHANGED_TO_AWAITING_VALIDATION,
            ),
            (
                OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(
                    OrderStatusChangedToStockConfirmedIntegrationEventPayload::new(Uuid::nil()),
                ),
                ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
            ),
            (
                OrderingIntegrationEvent::OrderStatusChangedToPaid(
                    OrderStatusChangedToPaidIntegrationEventPayload::new(Uuid::nil(), Vec::new()),
                ),
                ORDER_STATUS_CHANGED_TO_PAID,
            ),
            (
                OrderingIntegrationEvent::OrderShipped(OrderShippedIntegrationEventPayload::new(
                    Uuid::nil(),
                )),
                ORDER_SHIPPED,
            ),
            (
                OrderingIntegrationEvent::OrderCancelled(
                    OrderCancelledIntegrationEventPayload::new(Uuid::nil()),
                ),
                ORDER_CANCELLED,
            ),
        ];
        cases.iter().try_fold((), |(), (event, expected)| {
            let actual = event.event_name();
            check(actual == *expected, || {
                format!("expected {expected}, got {actual}")
            })
        })
    }

    #[test]
    fn all_event_names_covers_every_variant() -> Result<(), TestError> {
        let names = OrderingIntegrationEvent::all_event_names();
        check(names.len() == 6, || format!("len {}", names.len()))?;
        check(names.contains(&ORDER_STARTED), || {
            "missing OrderStarted".to_string()
        })?;
        check(names.contains(&ORDER_CANCELLED), || {
            "missing OrderCancelled".to_string()
        })
    }
}

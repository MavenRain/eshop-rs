//! Convert in-memory [`DomainEvent`]s into outbox rows.

use ordering_domain::DomainEvent;
use ordering_infrastructure::PendingEventLog;
use uuid::Uuid;

/// Project a single [`DomainEvent`] into an [`PendingEventLog`] row, ready
/// to be inserted alongside the aggregate change in the same transaction.
///
/// `transaction_id` is the application-supplied correlator that ties every
/// outbox row written in a single transaction together.
///
/// The `content` column carries a debug rendering of the event for v1;
/// a production deployment would route through `serde_json` once the
/// domain `DomainEvent` enum and its payload structs derive
/// `Serialize`/`Deserialize`.  See the project punchlist.
#[must_use]
pub fn domain_event_to_pending(event: &DomainEvent, transaction_id: Uuid) -> PendingEventLog {
    let event_id = Uuid::new_v4();
    let creation_time = chrono::Utc::now();
    let event_type_name = event_type_name(event).to_string();
    let content = format!("{event:?}");
    PendingEventLog::new(
        event_id,
        event_type_name,
        creation_time,
        content,
        transaction_id,
    )
}

fn event_type_name(event: &DomainEvent) -> &'static str {
    match event {
        DomainEvent::OrderStarted(_) => "OrderStarted",
        DomainEvent::OrderStatusChangedToAwaitingValidation(_) => {
            "OrderStatusChangedToAwaitingValidation"
        }
        DomainEvent::OrderStatusChangedToStockConfirmed(_) => "OrderStatusChangedToStockConfirmed",
        DomainEvent::OrderStatusChangedToPaid(_) => "OrderStatusChangedToPaid",
        DomainEvent::OrderShipped(_) => "OrderShipped",
        DomainEvent::OrderCancelled(_) => "OrderCancelled",
        DomainEvent::BuyerPaymentMethodVerified(_) => "BuyerPaymentMethodVerified",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordering_domain::{
        BuyerId, BuyerPaymentMethodVerifiedEvent, OrderCancelledEvent, OrderId, OrderShippedEvent,
        OrderStatusChangedToStockConfirmedEvent, PaymentMethodId,
    };

    use crate::error::Error;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    #[test]
    fn pending_carries_supplied_transaction_id() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let event = DomainEvent::OrderShipped(OrderShippedEvent::new(OrderId::new()));
        let pending = domain_event_to_pending(&event, txn);
        check(pending.transaction_id() == txn, || {
            "transaction_id mismatch".to_string()
        })?;
        check(pending.event_type_name() == "OrderShipped", || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn event_type_name_covers_each_variant() -> Result<(), Error> {
        let order_id = OrderId::new();
        let cases = [
            (
                DomainEvent::OrderShipped(OrderShippedEvent::new(order_id)),
                "OrderShipped",
            ),
            (
                DomainEvent::OrderCancelled(OrderCancelledEvent::new(order_id)),
                "OrderCancelled",
            ),
            (
                DomainEvent::OrderStatusChangedToStockConfirmed(
                    OrderStatusChangedToStockConfirmedEvent::new(order_id),
                ),
                "OrderStatusChangedToStockConfirmed",
            ),
            (
                DomainEvent::BuyerPaymentMethodVerified(BuyerPaymentMethodVerifiedEvent::new(
                    BuyerId::new(),
                    PaymentMethodId::new(),
                    order_id,
                )),
                "BuyerPaymentMethodVerified",
            ),
        ];
        cases.iter().try_fold((), |(), (event, expected)| {
            let actual = event_type_name(event);
            check(actual == *expected, || {
                format!("expected {expected}, got {actual}")
            })
        })
    }
}

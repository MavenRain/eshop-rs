//! Convert in-memory [`DomainEvent`]s into outbox rows.
//!
//! Each domain event is first translated to a wire-form
//! [`OrderingIntegrationEvent`] (lossy: card details and pure
//! in-process events are dropped) and then JSON-serialized into the
//! `content` column.  The publisher worker reads the column back
//! and forwards the integration event onto the bus.

use event_bus::IntegrationEvent;
use ordering_domain::DomainEvent;
use ordering_infrastructure::PendingEventLog;
use ordering_integration_events::from_domain_event;
use uuid::Uuid;

use crate::error::Error;

/// Project a single [`DomainEvent`] into an [`PendingEventLog`] row, ready
/// to be inserted alongside the aggregate change in the same transaction.
///
/// `transaction_id` is the application-supplied correlator that ties every
/// outbox row written in a single transaction together.
///
/// Returns [`Ok(None)`] when the domain event has no integration
/// counterpart (currently only `BuyerPaymentMethodVerified`); the caller
/// skips the outbox insert in that case.
///
/// # Errors
/// Returns [`Error::Json`] if the integration event fails to serialize.
pub fn domain_event_to_pending(
    event: &DomainEvent,
    transaction_id: Uuid,
) -> Result<Option<PendingEventLog>, Error> {
    from_domain_event(event)
        .map(|integration| {
            let content = serde_json::to_string(&integration)?;
            Ok(PendingEventLog::new(
                Uuid::new_v4(),
                integration.event_name().to_string(),
                chrono::Utc::now(),
                content,
                transaction_id,
            ))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordering_domain::{
        BuyerId, BuyerPaymentMethodVerifiedEvent, OrderCancelledEvent, OrderId, OrderShippedEvent,
        OrderStatusChangedToStockConfirmedEvent, PaymentMethodId,
    };
    use ordering_integration_events::{
        ORDER_CANCELLED, ORDER_SHIPPED, ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
        OrderingIntegrationEvent,
    };

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        match () {
            () if cond => Ok(()),
            () => Err(Error::Validation { reason: reason() }),
        }
    }

    #[test]
    fn pending_carries_supplied_transaction_id() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let event = DomainEvent::OrderShipped(OrderShippedEvent::new(OrderId::new()));
        let pending = domain_event_to_pending(&event, txn)?.ok_or_else(|| Error::Validation {
            reason: "OrderShipped should produce a pending row".to_string(),
        })?;
        check(pending.transaction_id() == txn, || {
            "transaction_id mismatch".to_string()
        })?;
        check(pending.event_type_name() == ORDER_SHIPPED, || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn buyer_payment_method_verified_skips_outbox() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let event = DomainEvent::BuyerPaymentMethodVerified(BuyerPaymentMethodVerifiedEvent::new(
            BuyerId::new(),
            PaymentMethodId::new(),
            OrderId::new(),
        ));
        let outcome = domain_event_to_pending(&event, txn)?;
        check(outcome.is_none(), || format!("got {outcome:?}"))
    }

    #[test]
    fn pending_content_round_trips_to_integration_event() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let order_id = OrderId::new();
        let event = DomainEvent::OrderCancelled(OrderCancelledEvent::new(order_id));
        let pending = domain_event_to_pending(&event, txn)?.ok_or_else(|| Error::Validation {
            reason: "OrderCancelled should produce a pending row".to_string(),
        })?;
        let parsed: OrderingIntegrationEvent =
            serde_json::from_str(pending.content()).map_err(|e| Error::Json {
                reason: e.to_string(),
            })?;
        let actual_order_id = match &parsed {
            OrderingIntegrationEvent::OrderCancelled(p) => p.order_id(),
            OrderingIntegrationEvent::OrderStarted(_)
            | OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(_)
            | OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(_)
            | OrderingIntegrationEvent::OrderStatusChangedToPaid(_)
            | OrderingIntegrationEvent::OrderShipped(_) => {
                return Err(Error::Validation {
                    reason: format!("unexpected variant {parsed:?}"),
                });
            }
        };
        check(actual_order_id == order_id.into_uuid(), || {
            format!("order_id {actual_order_id}")
        })?;
        check(pending.event_type_name() == ORDER_CANCELLED, || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn stock_confirmed_uses_wire_tag() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let event = DomainEvent::OrderStatusChangedToStockConfirmed(
            OrderStatusChangedToStockConfirmedEvent::new(OrderId::new()),
        );
        let pending = domain_event_to_pending(&event, txn)?.ok_or_else(|| Error::Validation {
            reason: "stock confirmed should produce a pending row".to_string(),
        })?;
        check(
            pending.event_type_name() == ORDER_STATUS_CHANGED_TO_STOCK_CONFIRMED,
            || format!("name {}", pending.event_type_name()),
        )
    }
}

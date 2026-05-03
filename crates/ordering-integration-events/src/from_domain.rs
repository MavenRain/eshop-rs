//! Translation from in-process domain events into wire-form
//! integration events.
//!
//! The mapping is intentionally lossy:
//!
//! - [`OrderStartedEvent`](ordering_domain::OrderStartedEvent) carries
//!   card details which never leave the Ordering service.  Only
//!   `order_id`, `user_id`, and `user_name` make it onto the wire.
//! - [`BuyerPaymentMethodVerifiedEvent`](ordering_domain::BuyerPaymentMethodVerifiedEvent)
//!   has no integration counterpart in upstream eShop and thus
//!   maps to [`None`].

use ordering_domain::DomainEvent;

use crate::event::OrderingIntegrationEvent;
use crate::payload::{
    OrderCancelledIntegrationEventPayload, OrderShippedIntegrationEventPayload,
    OrderStartedIntegrationEventPayload,
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload,
    OrderStatusChangedToStockConfirmedIntegrationEventPayload,
};
use crate::stock_item::OrderStockItem;

/// Project a domain event into its corresponding wire-form integration
/// event, if one exists.
///
/// Returns [`None`] for domain events that have no cross-service
/// counterpart (currently only
/// [`DomainEvent::BuyerPaymentMethodVerified`]).
#[must_use]
pub fn from_domain_event(event: &DomainEvent) -> Option<OrderingIntegrationEvent> {
    match event {
        DomainEvent::OrderStarted(payload) => Some(OrderingIntegrationEvent::OrderStarted(
            OrderStartedIntegrationEventPayload::new(
                payload.order_id().into_uuid(),
                payload.user_id().as_str().to_string(),
                payload.user_name().as_str().to_string(),
            ),
        )),
        DomainEvent::OrderStatusChangedToAwaitingValidation(payload) => Some(
            OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(
                OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                    payload.order_id().into_uuid(),
                    payload.order_items().iter().map(stock_item_from).collect(),
                ),
            ),
        ),
        DomainEvent::OrderStatusChangedToStockConfirmed(payload) => Some(
            OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(
                OrderStatusChangedToStockConfirmedIntegrationEventPayload::new(
                    payload.order_id().into_uuid(),
                ),
            ),
        ),
        DomainEvent::OrderStatusChangedToPaid(payload) => {
            Some(OrderingIntegrationEvent::OrderStatusChangedToPaid(
                OrderStatusChangedToPaidIntegrationEventPayload::new(
                    payload.order_id().into_uuid(),
                    payload.order_items().iter().map(stock_item_from).collect(),
                ),
            ))
        }
        DomainEvent::OrderShipped(payload) => Some(OrderingIntegrationEvent::OrderShipped(
            OrderShippedIntegrationEventPayload::new(payload.order_id().into_uuid()),
        )),
        DomainEvent::OrderCancelled(payload) => Some(OrderingIntegrationEvent::OrderCancelled(
            OrderCancelledIntegrationEventPayload::new(payload.order_id().into_uuid()),
        )),
        DomainEvent::BuyerPaymentMethodVerified(_) => None,
    }
}

fn stock_item_from(item: &ordering_domain::OrderItem) -> OrderStockItem {
    OrderStockItem::new(item.product_id().into(), item.units().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    use ordering_domain::{
        BuyerId, BuyerPaymentMethodVerifiedEvent, OrderCancelledEvent, OrderId, OrderShippedEvent,
        OrderStatusChangedToStockConfirmedEvent, PaymentMethodId,
    };

    #[derive(Debug)]
    struct TestError(String);

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl core::error::Error for TestError {}

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        match () {
            () if cond => Ok(()),
            () => Err(TestError(reason())),
        }
    }

    #[test]
    fn buyer_payment_method_verified_has_no_integration_event() -> Result<(), TestError> {
        let domain = DomainEvent::BuyerPaymentMethodVerified(BuyerPaymentMethodVerifiedEvent::new(
            BuyerId::new(),
            PaymentMethodId::new(),
            OrderId::new(),
        ));
        let outcome = from_domain_event(&domain);
        check(outcome.is_none(), || format!("got {outcome:?}"))
    }

    #[test]
    fn order_shipped_translates() -> Result<(), TestError> {
        let order_id = OrderId::new();
        let domain = DomainEvent::OrderShipped(OrderShippedEvent::new(order_id));
        let outcome = from_domain_event(&domain);
        let actual = outcome
            .as_ref()
            .and_then(|event| match event {
                OrderingIntegrationEvent::OrderShipped(p) => Some(p.order_id()),
                OrderingIntegrationEvent::OrderStarted(_)
                | OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(_)
                | OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(_)
                | OrderingIntegrationEvent::OrderStatusChangedToPaid(_)
                | OrderingIntegrationEvent::OrderCancelled(_) => None,
            })
            .unwrap_or_default();
        check(actual == order_id.into_uuid(), || format!("got {actual}"))
    }

    #[test]
    fn order_cancelled_translates() -> Result<(), TestError> {
        let order_id = OrderId::new();
        let domain = DomainEvent::OrderCancelled(OrderCancelledEvent::new(order_id));
        let outcome = from_domain_event(&domain);
        let actual = outcome
            .as_ref()
            .and_then(|event| match event {
                OrderingIntegrationEvent::OrderCancelled(p) => Some(p.order_id()),
                OrderingIntegrationEvent::OrderStarted(_)
                | OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(_)
                | OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(_)
                | OrderingIntegrationEvent::OrderStatusChangedToPaid(_)
                | OrderingIntegrationEvent::OrderShipped(_) => None,
            })
            .unwrap_or_default();
        check(actual == order_id.into_uuid(), || format!("got {actual}"))
    }

    #[test]
    fn stock_confirmed_translates() -> Result<(), TestError> {
        let order_id = OrderId::new();
        let domain = DomainEvent::OrderStatusChangedToStockConfirmed(
            OrderStatusChangedToStockConfirmedEvent::new(order_id),
        );
        let outcome = from_domain_event(&domain);
        let actual = outcome
            .as_ref()
            .and_then(|event| match event {
                OrderingIntegrationEvent::OrderStatusChangedToStockConfirmed(p) => {
                    Some(p.order_id())
                }
                OrderingIntegrationEvent::OrderStarted(_)
                | OrderingIntegrationEvent::OrderStatusChangedToAwaitingValidation(_)
                | OrderingIntegrationEvent::OrderStatusChangedToPaid(_)
                | OrderingIntegrationEvent::OrderShipped(_)
                | OrderingIntegrationEvent::OrderCancelled(_) => None,
            })
            .unwrap_or_default();
        check(actual == order_id.into_uuid(), || format!("got {actual}"))
    }
}

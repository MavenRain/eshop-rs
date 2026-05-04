//! Convert in-memory [`DomainEvent`]s into outbox rows.
//!
//! Each domain event is first translated to a wire-form
//! [`BasketIntegrationEvent`] and then JSON-serialized into the
//! `content` column.  The `basket-processor` worker reads the column
//! back and forwards the integration event onto the bus.
//!
//! Translation lives here rather than in `basket-integration-events`
//! so the dependency arrow stays one-directional: this crate (basket)
//! depends on `basket-integration-events`, not vice versa.

use basket_integration_events::{
    BasketIntegrationEvent, UserCheckoutAcceptedIntegrationEventPayload,
};
use event_bus::IntegrationEvent;
use ordering_integration_events::OrderStockItem;
use uuid::Uuid;

use crate::basket_item::BasketItem;
use crate::error::Error;
use crate::event::DomainEvent;
use crate::integration_event_log::PendingEventLog;

/// Project a single [`DomainEvent`] into a [`PendingEventLog`] row,
/// ready to be inserted alongside the aggregate change in the same
/// transaction.
///
/// `transaction_id` is the application-supplied correlator that ties
/// every outbox row written in a single transaction together.
///
/// Returns [`Ok(None)`] when the domain event has no integration
/// counterpart; the caller skips the outbox insert in that case.
/// Every basket domain variant currently maps to [`Some`], but the
/// signature is reserved for forward compatibility (parity with
/// `ordering-api::outbox::domain_event_to_pending`, where
/// `BuyerPaymentMethodVerified` returns [`None`]).
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

/// Project a domain event into its wire-form integration counterpart,
/// if one exists.
///
/// Currently every basket domain variant maps to [`Some`]; the
/// [`Option`] wrap is reserved for forward compatibility with future
/// in-process-only variants.
#[allow(clippy::unnecessary_wraps)]
#[must_use]
pub fn from_domain_event(event: &DomainEvent) -> Option<BasketIntegrationEvent> {
    match event {
        DomainEvent::CheckoutAccepted(payload) => {
            let stock_items: Vec<OrderStockItem> =
                payload.items().iter().map(stock_item_from).collect();
            Some(BasketIntegrationEvent::UserCheckoutAccepted(
                UserCheckoutAcceptedIntegrationEventPayload::new(
                    payload.customer_id().into_uuid(),
                    payload.request_id(),
                    stock_items,
                ),
            ))
        }
    }
}

fn stock_item_from(item: &BasketItem) -> OrderStockItem {
    OrderStockItem::new(item.product_id().into(), item.quantity().get())
}

#[cfg(test)]
mod tests {
    use super::*;

    use basket_integration_events::USER_CHECKOUT_ACCEPTED;
    use rust_decimal::Decimal;

    use crate::basket_item::{BasketItem, BasketItemId};
    use crate::customer::CustomerId;
    use crate::event::CheckoutAcceptedEvent;
    use crate::money::{Price, Quantity};
    use crate::product::ProductId;
    use crate::strings::ProductName;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_basket_item() -> Result<BasketItem, Error> {
        Ok(BasketItem::new(
            BasketItemId::new(),
            ProductId::from(42),
            ProductName::try_from(".NET Bot Hoodie")?,
            Price::new(Decimal::from(20))?,
            None,
            Quantity::new(3)?,
            None,
        ))
    }

    fn sample_domain_event() -> Result<DomainEvent, Error> {
        Ok(DomainEvent::CheckoutAccepted(CheckoutAcceptedEvent::new(
            CustomerId::new(),
            Uuid::new_v4(),
            vec![sample_basket_item()?],
        )))
    }

    #[test]
    fn pending_carries_supplied_transaction_id() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let pending = domain_event_to_pending(&sample_domain_event()?, txn)?.ok_or_else(|| {
            Error::Validation {
                reason: "CheckoutAccepted should produce a pending row".to_string(),
            }
        })?;
        check(pending.transaction_id() == txn, || {
            "transaction_id mismatch".to_string()
        })?;
        check(pending.event_type_name() == USER_CHECKOUT_ACCEPTED, || {
            format!("name {}", pending.event_type_name())
        })
    }

    #[test]
    fn pending_content_round_trips_to_integration_event() -> Result<(), Error> {
        let txn = Uuid::new_v4();
        let domain = sample_domain_event()?;
        let pending = domain_event_to_pending(&domain, txn)?.ok_or_else(|| Error::Validation {
            reason: "CheckoutAccepted should produce a pending row".to_string(),
        })?;
        let parsed: BasketIntegrationEvent = serde_json::from_str(pending.content())?;
        let BasketIntegrationEvent::UserCheckoutAccepted(payload) = &parsed;
        check(payload.order_stock_items().len() == 1, || {
            format!("items {}", payload.order_stock_items().len())
        })?;
        let item = payload
            .order_stock_items()
            .first()
            .ok_or(Error::Validation {
                reason: "empty items".to_string(),
            })?;
        check(item.product_id() == 42, || {
            format!("product_id {}", item.product_id())
        })?;
        check(item.units() == 3, || format!("units {}", item.units()))
    }

    #[test]
    fn from_domain_event_translates_checkout_accepted() -> Result<(), Error> {
        let domain = sample_domain_event()?;
        let outcome = from_domain_event(&domain);
        check(outcome.is_some(), || format!("got {outcome:?}"))
    }
}

//! Dispatch handler for the events the catalog consumes from the
//! ordering bus.
//!
//! For the wire-validation slice the handler bodies are observation
//! only: they log the received event and ack.  Real bodies (stock
//! reservation on `AwaitingValidation`, commit on `Paid`, release on
//! cancellation) land once the catalog `ProductId` has been aligned
//! with the ordering wire format.

use comp_cat_rs::effect::io::Io;
use event_bus::Error as BusError;
use ordering_integration_events::{
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload,
};

use crate::consumed_event::CatalogConsumedOrderingEvent;

/// Handler suitable for [`EventBusSubscriber::subscribe`](event_bus::EventBusSubscriber::subscribe).
///
/// Returns an [`Io`] so the bus can drive it on its owned runtime.
/// The current bodies log to stderr and return `Ok(())`; the bus
/// then acks the message.
#[must_use]
pub fn handle(event: CatalogConsumedOrderingEvent) -> Io<BusError, ()> {
    Io::suspend(move || match event {
        CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation(payload) => {
            handle_awaiting_validation(&payload)
        }
        CatalogConsumedOrderingEvent::OrderStatusChangedToPaid(payload) => handle_paid(&payload),
    })
}

// `Result<(), BusError>` rather than the simpler `()`: forward-compat
// for the real handler bodies (stock decrement, etc.), which will
// fail on persistence errors.  The `#[allow]` keeps clippy quiet
// while every variant is still a no-op.
#[allow(clippy::unnecessary_wraps)]
fn handle_awaiting_validation(
    payload: &OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
) -> Result<(), BusError> {
    eprintln!(
        "[catalog-subscribers] OrderStatusChangedToAwaitingValidation: order_id={} stock_items={}",
        payload.order_id(),
        payload.order_stock_items().len()
    );
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn handle_paid(payload: &OrderStatusChangedToPaidIntegrationEventPayload) -> Result<(), BusError> {
    eprintln!(
        "[catalog-subscribers] OrderStatusChangedToPaid: order_id={} stock_items={}",
        payload.order_id(),
        payload.order_stock_items().len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use ordering_integration_events::OrderStockItem;
    use uuid::Uuid;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), BusError> {
        match () {
            () if cond => Ok(()),
            () => Err(BusError::Subscribe { reason: reason() }),
        }
    }

    #[test]
    fn handle_awaiting_validation_returns_ok() -> Result<(), BusError> {
        let event = CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation(
            OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
                Uuid::nil(),
                vec![OrderStockItem::new(1, 1)],
            ),
        );
        let outcome = handle(event).run();
        check(outcome.is_ok(), || format!("got {outcome:?}"))
    }

    #[test]
    fn handle_paid_returns_ok() -> Result<(), BusError> {
        let event = CatalogConsumedOrderingEvent::OrderStatusChangedToPaid(
            OrderStatusChangedToPaidIntegrationEventPayload::new(
                Uuid::nil(),
                vec![OrderStockItem::new(2, 4)],
            ),
        );
        let outcome = handle(event).run();
        check(outcome.is_ok(), || format!("got {outcome:?}"))
    }
}

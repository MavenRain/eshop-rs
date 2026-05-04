//! Dispatch handler for the events the ordering bounded context
//! consumes from the basket bus.
//!
//! For the wire-validation slice the handler body is observation
//! only: it logs the received event and acks.  The real body (mint
//! an order from the carried basket snapshot, pre-validate stock
//! against catalog, hand off to the ordering domain) lands once the
//! catalog/ordering `ProductId` alignment slice reconciles the
//! wire format with the catalog domain types.

use basket_integration_events::UserCheckoutAcceptedIntegrationEventPayload;
use comp_cat_rs::effect::io::Io;
use event_bus::Error as BusError;

use crate::consumed_event::OrderingConsumedBasketEvent;

/// Handler suitable for [`EventBusSubscriber::subscribe`](event_bus::EventBusSubscriber::subscribe).
///
/// Returns an [`Io`] so the bus can drive it on its owned runtime.
/// The current body logs to stderr and returns `Ok(())`; the bus
/// then acks the message.
#[must_use]
pub fn handle(event: OrderingConsumedBasketEvent) -> Io<BusError, ()> {
    Io::suspend(move || match event {
        OrderingConsumedBasketEvent::UserCheckoutAccepted(payload) => {
            handle_user_checkout_accepted(&payload)
        }
    })
}

// `Result<(), BusError>` rather than the simpler `()`: forward-compat
// for the real handler body, which will fail on persistence errors.
// The `#[allow]` keeps clippy quiet while the variant is still a
// no-op.
#[allow(clippy::unnecessary_wraps)]
fn handle_user_checkout_accepted(
    payload: &UserCheckoutAcceptedIntegrationEventPayload,
) -> Result<(), BusError> {
    eprintln!(
        "[ordering-subscribers] UserCheckoutAccepted: customer_id={} request_id={} stock_items={}",
        payload.customer_id(),
        payload.request_id(),
        payload.order_stock_items().len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), BusError> {
        if cond {
            Ok(())
        } else {
            Err(BusError::Subscribe { reason: reason() })
        }
    }

    #[test]
    fn handle_user_checkout_accepted_returns_ok() -> Result<(), BusError> {
        let event = OrderingConsumedBasketEvent::UserCheckoutAccepted(
            UserCheckoutAcceptedIntegrationEventPayload::new(Uuid::nil(), Uuid::max(), Vec::new()),
        );
        let outcome = handle(event).run();
        check(outcome.is_ok(), || format!("got {outcome:?}"))
    }
}

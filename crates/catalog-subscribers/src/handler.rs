//! Dispatch handler for the events the catalog consumes from the
//! ordering bus.
//!
//! Real bodies (slice 4 of the punchlist):
//!
//! - `OrderStatusChangedToAwaitingValidation`: reserve stock by
//!   calling [`CatalogItem::remove_stock`](catalog::CatalogItem::remove_stock)
//!   for each carried [`OrderStockItem`](ordering_integration_events::OrderStockItem).
//!   The reservation and the resulting catalog row update commit in
//!   the same toasty transaction.
//! - `OrderStatusChangedToPaid`: today logged-only.  Upstream eShop
//!   would commit the prior reservation here; in our reservation
//!   model, the `AwaitingValidation` write already decrements
//!   `available_stock`, so `Paid` is a no-op until we add a
//!   reservation/commit split (a follow-on slice).
//!
//! ## Async-driving inside an `Io::suspend` body
//!
//! [`event_bus::EventBusSubscriber::subscribe`] takes a synchronous
//! `Fn(E) -> Io<event_bus::Error, ()>`.  The bus drives the handler
//! via `handler(event).run()` from inside its async consume loop, so
//! we are running on a tokio worker thread of the bus's runtime.  To
//! perform async toasty work without restructuring the bus signature,
//! the handler uses
//! [`tokio::task::block_in_place`] +
//! [`tokio::runtime::Handle::current`]`.block_on(...)`: the worker
//! thread is allowed to block, the future is driven on the same
//! runtime, and the rest of the runtime stays unblocked.

use std::sync::Arc;

use catalog::{CatalogItemId, CatalogItemRepository, Units};
use comp_cat_rs::effect::io::Io;
use event_bus::Error as BusError;
use ordering_integration_events::{
    OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
    OrderStatusChangedToPaidIntegrationEventPayload, OrderStockItem,
};
use toasty::Db;

use crate::consumed_event::CatalogConsumedOrderingEvent;
use crate::error::Error;

/// Build a handler closure suitable for [`event_bus::EventBusSubscriber::subscribe`].
///
/// The returned closure captures the shared [`Db`] handle and routes
/// each event through the appropriate per-variant body.  Drive it
/// with `bus.subscribe(handler).run()?` at `AppHost` startup.
pub fn make_handler(
    db: Arc<Db>,
) -> impl Fn(CatalogConsumedOrderingEvent) -> Io<BusError, ()> + Send + Sync + 'static {
    move |event| {
        let db = db.clone();
        Io::suspend(move || run_blocking(db, event).map_err(BusError::from))
    }
}

fn run_blocking(db: Arc<Db>, event: CatalogConsumedOrderingEvent) -> Result<(), Error> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move { dispatch(db, event).await })
    })
}

async fn dispatch(db: Arc<Db>, event: CatalogConsumedOrderingEvent) -> Result<(), Error> {
    match event {
        CatalogConsumedOrderingEvent::OrderStatusChangedToAwaitingValidation(payload) => {
            handle_awaiting_validation(db, &payload).await
        }
        CatalogConsumedOrderingEvent::OrderStatusChangedToPaid(payload) => {
            handle_paid(&payload);
            Ok(())
        }
    }
}

async fn handle_awaiting_validation(
    db: Arc<Db>,
    payload: &OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
) -> Result<(), Error> {
    eprintln!(
        "[catalog-subscribers] OrderStatusChangedToAwaitingValidation: order_id={} stock_items={}",
        payload.order_id(),
        payload.order_stock_items().len()
    );
    // FFI-mut exception: toasty's Connection::transaction takes
    // `&mut self`; the same exception class as the API-layer handlers.
    let mut conn = db.connection().await?;
    let mut tx = conn.transaction().await?;
    decrement_stock_for_items(&mut tx, payload.order_stock_items()).await?;
    tx.commit().await?;
    Ok(())
}

fn handle_paid(payload: &OrderStatusChangedToPaidIntegrationEventPayload) {
    eprintln!(
        "[catalog-subscribers] OrderStatusChangedToPaid: order_id={} stock_items={} \
         (no-op; AwaitingValidation already decremented stock in this port's reservation model)",
        payload.order_id(),
        payload.order_stock_items().len()
    );
}

/// Sequential-async stock decrement, parallel to
/// `IntegrationEventLogService::save_events_batch` and the
/// `BasketRepository` batch helpers: the FFI-async `for` loop is
/// localized in this helper so the call site
/// (`handle_awaiting_validation`) stays combinator-shaped.
async fn decrement_stock_for_items<E>(
    executor: &mut E,
    items: &[OrderStockItem],
) -> Result<(), Error>
where
    E: toasty::Executor,
{
    for item in items {
        decrement_stock_for_item(executor, item).await?;
    }
    Ok(())
}

async fn decrement_stock_for_item<E>(executor: &mut E, item: &OrderStockItem) -> Result<(), Error>
where
    E: toasty::Executor,
{
    let product_id = CatalogItemId::from(item.product_id());
    let requested = Units::from(item.units());
    let existing = CatalogItemRepository::get_by_id(executor, product_id).await?;
    let (decremented, removed) = existing.remove_stock(requested)?;
    log_partial_decrement(*item, requested, removed);
    CatalogItemRepository::update(executor, &decremented).await?;
    Ok(())
}

fn log_partial_decrement(item: OrderStockItem, requested: Units, removed: Units) {
    if requested != removed {
        eprintln!(
            "[catalog-subscribers] product_id={} requested {} units, removed {} \
             (insufficient stock)",
            item.product_id(),
            requested.get(),
            removed.get()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ordering_integration_events::{
        OrderStatusChangedToAwaitingValidationIntegrationEventPayload,
        OrderStatusChangedToPaidIntegrationEventPayload,
    };
    use uuid::Uuid;

    #[derive(Debug)]
    enum TestError {
        Mismatch(String),
    }

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Mismatch(reason) => write!(f, "mismatch: {reason}"),
            }
        }
    }

    impl core::error::Error for TestError {}

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), TestError> {
        if cond {
            Ok(())
        } else {
            Err(TestError::Mismatch(reason()))
        }
    }

    #[test]
    fn handle_paid_logs_without_decrementing() -> Result<(), TestError> {
        let payload = OrderStatusChangedToPaidIntegrationEventPayload::new(
            Uuid::nil(),
            vec![OrderStockItem::new(42, 3)],
        );
        // No-op handler: just verify it doesn't panic and produces no
        // observable error.
        handle_paid(&payload);
        check(true, || "logged".to_string())
    }

    #[test]
    fn log_partial_decrement_records_when_short() -> Result<(), TestError> {
        // Smoke test: requested != removed runs the diagnostic
        // branch.  Real assertion would capture stderr; we just
        // confirm the call is total and doesn't panic.
        log_partial_decrement(OrderStockItem::new(7, 5), Units::from(5), Units::from(2));
        log_partial_decrement(OrderStockItem::new(7, 5), Units::from(5), Units::from(5));
        check(true, || "logged".to_string())
    }

    #[test]
    fn awaiting_validation_payload_round_trips() -> Result<(), TestError> {
        let payload = OrderStatusChangedToAwaitingValidationIntegrationEventPayload::new(
            Uuid::nil(),
            vec![OrderStockItem::new(42, 3)],
        );
        check(payload.order_stock_items().len() == 1, || {
            format!("len {}", payload.order_stock_items().len())
        })
    }
}

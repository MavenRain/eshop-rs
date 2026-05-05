//! Dispatch handler for the events the ordering bounded context
//! consumes from the basket bus.
//!
//! On `UserCheckoutAccepted` we mint an [`Order`] from the carried
//! basket snapshot.  The order is persisted via
//! [`OrderRepository::add`], any [`DomainEvent`]s emitted by the
//! constructor are projected through
//! [`from_domain_event`](ordering_integration_events::from_domain_event),
//! JSON-serialized and saved through the ordering outbox via
//! [`IntegrationEventLogService::save_events_batch`].  The aggregate
//! write and the outbox writes commit in the same toasty transaction.
//!
//! ## Sentinel fields
//!
//! Upstream eShop's `UserCheckoutAcceptedIntegrationEvent` carries
//! shipping address and card details captured by the basket UI.  Our
//! port does not yet have an identity bounded context to source those
//! from, so the subscriber fills the address/card portion of the new
//! order with sentinel `"TBD"` strings.  When the identity context
//! lands, the basket integration event will grow those fields and the
//! subscriber will read them off the payload directly.
//!
//! ## Async-driving inside an `Io::suspend` body
//!
//! Same approach as `catalog-subscribers`: the bus drives the handler
//! synchronously via `handler(event).run()`, so the body uses
//! [`tokio::task::block_in_place`] +
//! [`tokio::runtime::Handle::current`]`.block_on(...)` to drive the
//! toasty future on the bus's owned multi-threaded runtime.

use std::sync::Arc;

use basket_integration_events::{BasketSnapshotItem, UserCheckoutAcceptedIntegrationEventPayload};
use chrono::{Duration, Utc};
use comp_cat_rs::effect::io::Io;
use event_bus::{Error as BusError, IntegrationEvent};
use ordering_domain::{
    Address, CardExpiration, CardHolderName, CardNumber, CardType, City, Country, Discount,
    DomainEvent, Money, Order, OrderId, OrderItem, OrderItemId, PictureUrl, ProductId, ProductName,
    SecurityNumber, State, Street, UnitPrice, Units, UserId, UserName, ZipCode,
};
use ordering_infrastructure::{IntegrationEventLogService, OrderRepository, PendingEventLog};
use ordering_integration_events::from_domain_event;
use rust_decimal::Decimal;
use toasty::Db;
use uuid::Uuid;

use crate::consumed_event::OrderingConsumedBasketEvent;
use crate::error::Error;

const SENTINEL_STRING: &str = "TBD";

/// Build a handler closure suitable for [`event_bus::EventBusSubscriber::subscribe`].
///
/// The returned closure captures the shared [`Db`] handle and routes
/// each event through the appropriate per-variant body.  Drive it
/// with `bus.subscribe(handler).run()?` at `AppHost` startup.
pub fn make_handler(
    db: Arc<Db>,
) -> impl Fn(OrderingConsumedBasketEvent) -> Io<BusError, ()> + Send + Sync + 'static {
    move |event| {
        let db = db.clone();
        Io::suspend(move || run_blocking(db, event).map_err(BusError::from))
    }
}

fn run_blocking(db: Arc<Db>, event: OrderingConsumedBasketEvent) -> Result<(), Error> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move { dispatch(db, event).await })
    })
}

async fn dispatch(db: Arc<Db>, event: OrderingConsumedBasketEvent) -> Result<(), Error> {
    match event {
        OrderingConsumedBasketEvent::UserCheckoutAccepted(payload) => {
            handle_user_checkout_accepted(db, &payload).await
        }
    }
}

async fn handle_user_checkout_accepted(
    db: Arc<Db>,
    payload: &UserCheckoutAcceptedIntegrationEventPayload,
) -> Result<(), Error> {
    eprintln!(
        "[ordering-subscribers] UserCheckoutAccepted: customer_id={} request_id={} basket_items={}",
        payload.customer_id(),
        payload.request_id(),
        payload.basket_items().len()
    );
    let order = build_order(payload)?;
    let (cleaned_order, events) = order.take_events();
    // FFI-mut exception: toasty's Connection::transaction takes
    // `&mut self`; same exception class as the API-layer handlers.
    let mut conn = db.connection().await?;
    let mut tx = conn.transaction().await?;
    OrderRepository::add(&mut tx, &cleaned_order).await?;
    let transaction_id = Uuid::new_v4();
    let pendings: Vec<PendingEventLog> = events
        .iter()
        .filter_map(|event| project_to_pending(event, transaction_id).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    IntegrationEventLogService::save_events_batch(&mut tx, pendings).await?;
    tx.commit().await?;
    Ok(())
}

/// Build a fully-populated [`Order`] from the basket snapshot, with
/// sentinel address and card fields (see module-level doc).
fn build_order(payload: &UserCheckoutAcceptedIntegrationEventPayload) -> Result<Order, Error> {
    let user_id_string = payload.customer_id().to_string();
    let order = Order::submit(
        OrderId::new(),
        UserId::try_from(user_id_string.clone())?,
        UserName::try_from(user_id_string)?,
        sentinel_address()?,
        CardType::Visa,
        CardNumber::try_from(SENTINEL_STRING)?,
        SecurityNumber::try_from(SENTINEL_STRING)?,
        CardHolderName::try_from(SENTINEL_STRING)?,
        CardExpiration::new(Utc::now() + Duration::days(365), Utc::now())?,
        None,
        None,
    );
    payload
        .basket_items()
        .iter()
        .try_fold(order, append_basket_snapshot_item)
}

fn append_basket_snapshot_item(
    order: Order,
    snapshot: &BasketSnapshotItem,
) -> Result<Order, Error> {
    let order_item = build_order_item(snapshot)?;
    Ok(order.add_order_item(order_item)?)
}

fn build_order_item(snapshot: &BasketSnapshotItem) -> Result<OrderItem, Error> {
    let units = Units::new(snapshot.units())?;
    let unit_price = UnitPrice::from(Money::from(snapshot.unit_price()));
    let discount = Discount::new(Money::from(Decimal::ZERO))?;
    let product_name = ProductName::try_from(snapshot.product_name().to_string())?;
    let picture_url = PictureUrl::try_from(snapshot.picture_url().unwrap_or(SENTINEL_STRING))?;
    Ok(OrderItem::new(
        OrderItemId::new(),
        ProductId::from(snapshot.product_id()),
        product_name,
        picture_url,
        unit_price,
        discount,
        units,
    )?)
}

fn sentinel_address() -> Result<Address, Error> {
    Ok(Address::new(
        Street::try_from(SENTINEL_STRING)?,
        City::try_from(SENTINEL_STRING)?,
        State::try_from(SENTINEL_STRING)?,
        Country::try_from(SENTINEL_STRING)?,
        ZipCode::try_from(SENTINEL_STRING)?,
    ))
}

/// Mirror of `ordering-api::outbox::domain_event_to_pending` so the
/// subscriber doesn't take a dep on the API crate just for an outbox
/// projection.  Returns `Ok(None)` for domain events that have no
/// integration counterpart (currently only
/// `BuyerPaymentMethodVerified`).
fn project_to_pending(
    event: &DomainEvent,
    transaction_id: Uuid,
) -> Result<Option<PendingEventLog>, Error> {
    from_domain_event(event)
        .map(|integration| {
            let content = serde_json::to_string(&integration)?;
            Ok(PendingEventLog::new(
                Uuid::new_v4(),
                integration.event_name().to_string(),
                Utc::now(),
                content,
                transaction_id,
            ))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Handler { reason: reason() })
        }
    }

    fn sample_payload() -> UserCheckoutAcceptedIntegrationEventPayload {
        UserCheckoutAcceptedIntegrationEventPayload::new(
            Uuid::nil(),
            Uuid::max(),
            vec![BasketSnapshotItem::new(
                42,
                "Hoodie".to_string(),
                Decimal::from(20),
                Some("hoodie.png".to_string()),
                3,
            )],
        )
    }

    #[test]
    fn build_order_carries_basket_items() -> Result<(), Error> {
        let order = build_order(&sample_payload())?;
        check(order.order_items().len() == 1, || {
            format!("expected 1 item, got {}", order.order_items().len())
        })?;
        let item = order.order_items().first().ok_or_else(|| Error::Handler {
            reason: "empty order_items".to_string(),
        })?;
        check(i32::from(item.product_id()) == 42, || {
            format!("product_id {}", i32::from(item.product_id()))
        })?;
        check(item.product_name().as_str() == "Hoodie", || {
            format!("name {}", item.product_name().as_str())
        })?;
        check(item.units().get() == 3, || {
            format!("units {}", item.units().get())
        })
    }

    #[test]
    fn build_order_emits_order_started_event() -> Result<(), Error> {
        let order = build_order(&sample_payload())?;
        let event_count = order.domain_events().len();
        check(event_count >= 1, || format!("event_count {event_count}"))
    }

    #[test]
    fn build_order_uses_missing_picture_url_sentinel() -> Result<(), Error> {
        let payload = UserCheckoutAcceptedIntegrationEventPayload::new(
            Uuid::nil(),
            Uuid::max(),
            vec![BasketSnapshotItem::new(
                7,
                "Mug".to_string(),
                Decimal::from(5),
                None,
                1,
            )],
        );
        let order = build_order(&payload)?;
        let item = order.order_items().first().ok_or_else(|| Error::Handler {
            reason: "empty order_items".to_string(),
        })?;
        check(item.picture_url().as_str() == SENTINEL_STRING, || {
            format!("picture {}", item.picture_url().as_str())
        })
    }

    #[test]
    fn build_order_rejects_zero_units() -> Result<(), Error> {
        let payload = UserCheckoutAcceptedIntegrationEventPayload::new(
            Uuid::nil(),
            Uuid::max(),
            vec![BasketSnapshotItem::new(
                7,
                "Mug".to_string(),
                Decimal::from(5),
                None,
                0,
            )],
        );
        let outcome = build_order(&payload);
        check(matches!(outcome, Err(Error::Domain { .. })), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn project_to_pending_emits_row_for_order_started() -> Result<(), Error> {
        let order = build_order(&sample_payload())?;
        let (_cleaned, events) = order.take_events();
        let txn = Uuid::new_v4();
        let pendings: Vec<_> = events
            .iter()
            .filter_map(|event| project_to_pending(event, txn).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        check(!pendings.is_empty(), || {
            format!("pendings {}", pendings.len())
        })
    }
}

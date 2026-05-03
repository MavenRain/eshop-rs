//! `PUT /orders/ship` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use ordering_infrastructure::{IntegrationEventLogService, OrderRepository};
use uuid::Uuid;

use crate::error::Error;
use crate::outbox::domain_event_to_pending;
use crate::request::ShipOrderRequest;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<ShipOrderRequest>,
) -> Result<StatusCode, Error> {
    let order_id = request.order_id();

    // FFI-mut exception: see `create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;

    let order = OrderRepository::get_by_id(&mut tx, order_id).await?;
    let shipped = order.set_shipped_status()?;
    OrderRepository::update_status(
        &mut tx,
        shipped.id(),
        shipped.order_status(),
        shipped.description().as_str(),
    )
    .await?;
    let (_drained, events) = shipped.take_events();
    let transaction_id = Uuid::new_v4();
    // `filter_map` + `Result::transpose` drops domain events that have no
    // integration counterpart (currently `BuyerPaymentMethodVerified`)
    // and propagates serialization errors; `save_events_batch` then
    // performs the sequential insert behind a single API call.
    let pendings: Vec<ordering_infrastructure::PendingEventLog> = events
        .iter()
        .filter_map(|event| domain_event_to_pending(event, transaction_id).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    IntegrationEventLogService::save_events_batch(&mut tx, pendings).await?;
    tx.commit().await?;

    Ok(StatusCode::OK)
}

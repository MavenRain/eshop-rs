//! `PUT /orders/cancel` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use ordering_infrastructure::{IntegrationEventLogService, OrderRepository};
use uuid::Uuid;

use crate::error::Error;
use crate::outbox::domain_event_to_pending;
use crate::request::CancelOrderRequest;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<CancelOrderRequest>,
) -> Result<StatusCode, Error> {
    let order_id = request.order_id();

    // FFI-mut exception: see `create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;

    let order = OrderRepository::get_by_id(&mut tx, order_id).await?;
    let cancelled = order.set_cancelled_status()?;
    OrderRepository::update_status(
        &mut tx,
        cancelled.id(),
        cancelled.order_status(),
        cancelled.description().as_str(),
    )
    .await?;
    let (_drained, events) = cancelled.take_events();
    let transaction_id = Uuid::new_v4();
    // Sequential async insert; FFI exception (see ordering-infrastructure).
    for event in &events {
        let pending = domain_event_to_pending(event, transaction_id);
        IntegrationEventLogService::save_event(&mut tx, pending).await?;
    }
    tx.commit().await?;

    Ok(StatusCode::OK)
}

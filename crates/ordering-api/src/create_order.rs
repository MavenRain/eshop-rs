//! `POST /orders` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use ordering_infrastructure::{IntegrationEventLogService, OrderRepository};
use uuid::Uuid;

use crate::error::Error;
use crate::outbox::domain_event_to_pending;
use crate::request::CreateOrderRequest;
use crate::response::CreateOrderResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<CreateOrderResponse>), Error> {
    let order = request.try_into_order(chrono::Utc::now())?;
    let order_id = order.id();
    let (cleaned_order, events) = order.take_events();

    // FFI-mut exception: toasty's Connection::transaction takes &mut self,
    // and Transaction is then used as `&mut Executor` by the repositories.
    // Both bindings are unavoidable until toasty exposes a non-mutable
    // borrow form; they are scoped to a single handler.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    OrderRepository::add(&mut tx, &cleaned_order).await?;
    let transaction_id = Uuid::new_v4();
    write_events(&mut tx, &events, transaction_id).await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateOrderResponse::new(order_id)),
    ))
}

async fn write_events<E>(
    executor: &mut E,
    events: &[ordering_domain::DomainEvent],
    transaction_id: Uuid,
) -> Result<(), Error>
where
    E: toasty::Executor,
{
    // Project events into outbox rows using `filter_map` + `Result::transpose`:
    // domain events without an integration counterpart (currently
    // `BuyerPaymentMethodVerified`) drop out as `None`, errors propagate
    // through the `collect::<Result<_, _>>()`.  The sequential async insert
    // is encapsulated by `save_events_batch` so this function stays
    // combinator-only.
    let pendings: Vec<ordering_infrastructure::PendingEventLog> = events
        .iter()
        .filter_map(|event| domain_event_to_pending(event, transaction_id).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    IntegrationEventLogService::save_events_batch(executor, pendings).await?;
    Ok(())
}

//! `PUT /api/catalog/items/:id` handler.
//!
//! Loads the existing item, applies the request, and emits a
//! `ProductPriceChanged` outbox row inside the same toasty
//! transaction iff the price changed.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::error::Error;
use crate::integration_event_log::{IntegrationEventLogService, PendingEventLog};
use crate::item::CatalogItemId;
use crate::outbox::domain_event_to_pending;
use crate::repository::CatalogItemRepository;
use crate::request::UpdateItemRequest;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateItemRequest>,
) -> Result<StatusCode, Error> {
    let item_id = CatalogItemId::from(id);

    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;

    let existing = CatalogItemRepository::get_by_id(&mut tx, item_id).await?;
    let updated = request.apply_to(existing)?;
    let (cleaned, events) = updated.take_events();

    CatalogItemRepository::update(&mut tx, &cleaned).await?;

    // `filter_map` + `Result::transpose` drops domain events that have no
    // integration counterpart and propagates serialization errors;
    // `save_events_batch` then performs the sequential insert behind a
    // single API call.
    let transaction_id = Uuid::new_v4();
    let pendings: Vec<PendingEventLog> = events
        .iter()
        .filter_map(|event| domain_event_to_pending(event, transaction_id).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    IntegrationEventLogService::save_events_batch(&mut tx, pendings).await?;

    tx.commit().await?;
    Ok(StatusCode::OK)
}

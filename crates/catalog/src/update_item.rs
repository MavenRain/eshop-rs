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
use crate::integration_event_log::IntegrationEventLogService;
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

    // `CatalogItem::apply_update` emits at most one `ProductPriceChanged`
    // event per call, so the outbox write is a single optional projection
    // rather than an iteration.  If the catalog domain grows multi-event
    // updates later, swap this for a fold-style helper.
    let transaction_id = Uuid::new_v4();
    let pending = events
        .first()
        .map(|event| domain_event_to_pending(event, transaction_id));
    match pending {
        None => Ok(()),
        Some(p) => IntegrationEventLogService::save_event(&mut tx, p).await,
    }?;

    tx.commit().await?;
    Ok(StatusCode::OK)
}

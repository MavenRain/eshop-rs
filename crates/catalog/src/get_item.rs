//! `GET /api/catalog/items/:id` handler.

use axum::Json;
use axum::extract::{Path, State};

use crate::error::Error;
use crate::item::CatalogItemId;
use crate::repository::CatalogItemRepository;
use crate::response::CatalogItemResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<CatalogItemResponse>, Error> {
    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let item = CatalogItemRepository::get_by_id(&mut tx, CatalogItemId::from(id)).await?;
    tx.commit().await?;
    Ok(Json(CatalogItemResponse::from(&item)))
}

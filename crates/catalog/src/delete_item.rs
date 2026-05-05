//! `DELETE /api/catalog/items/:id` handler.

use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::error::Error;
use crate::item::CatalogItemId;
use crate::repository::CatalogItemRepository;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, Error> {
    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    CatalogItemRepository::delete_by_id(&mut tx, CatalogItemId::from(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

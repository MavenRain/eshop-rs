//! `GET /api/catalog/kinds` handler.

use axum::Json;
use axum::extract::State;

use crate::error::Error;
use crate::repository::CatalogKindRepository;
use crate::response::CatalogKindResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
) -> Result<Json<Vec<CatalogKindResponse>>, Error> {
    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let kinds = CatalogKindRepository::list_all(&mut tx).await?;
    tx.commit().await?;

    let response: Vec<CatalogKindResponse> = kinds.iter().map(CatalogKindResponse::from).collect();
    Ok(Json(response))
}

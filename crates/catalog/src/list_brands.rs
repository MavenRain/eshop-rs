//! `GET /api/catalog/brands` handler.

use axum::Json;
use axum::extract::State;

use crate::error::Error;
use crate::repository::CatalogBrandRepository;
use crate::response::CatalogBrandResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
) -> Result<Json<Vec<CatalogBrandResponse>>, Error> {
    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let brands = CatalogBrandRepository::list_all(&mut tx).await?;
    tx.commit().await?;

    let response: Vec<CatalogBrandResponse> =
        brands.iter().map(CatalogBrandResponse::from).collect();
    Ok(Json(response))
}

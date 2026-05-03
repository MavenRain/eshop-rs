//! `POST /api/catalog/brands` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::Error;
use crate::repository::CatalogBrandRepository;
use crate::request::CreateBrandRequest;
use crate::response::CreatedResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<CreateBrandRequest>,
) -> Result<(StatusCode, Json<CreatedResponse>), Error> {
    let brand = request.try_into_brand()?;
    let brand_id = brand.id();

    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    CatalogBrandRepository::add(&mut tx, &brand).await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedResponse::new(brand_id.into_uuid())),
    ))
}

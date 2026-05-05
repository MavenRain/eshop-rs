//! `POST /api/catalog/kinds` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::Error;
use crate::repository::CatalogKindRepository;
use crate::request::CreateKindRequest;
use crate::response::CreatedResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<CreateKindRequest>,
) -> Result<(StatusCode, Json<CreatedResponse<uuid::Uuid>>), Error> {
    let kind = request.try_into_kind()?;
    let kind_id = kind.id();

    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    CatalogKindRepository::add(&mut tx, &kind).await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedResponse::new(kind_id.into_uuid())),
    ))
}

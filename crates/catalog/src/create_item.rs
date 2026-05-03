//! `POST /api/catalog/items` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::Error;
use crate::repository::CatalogItemRepository;
use crate::request::CreateItemRequest;
use crate::response::CreatedResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Json(request): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<CreatedResponse>), Error> {
    let item = request.try_into_item()?;
    let item_id = item.id();

    // FFI-mut exception: toasty's Connection::transaction takes &mut
    // self, and Transaction is then used as `&mut Executor` by the
    // repository.  See `ordering-api::create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    CatalogItemRepository::add(&mut tx, &item).await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedResponse::new(item_id.into_uuid())),
    ))
}

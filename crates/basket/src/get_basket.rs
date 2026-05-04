//! `GET /api/basket/:customer_id` handler.

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::response::BasketResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<BasketResponse>, Error> {
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let basket =
        BasketRepository::get_by_customer_id(&mut tx, CustomerId::from(customer_id)).await?;
    tx.commit().await?;
    Ok(Json(BasketResponse::from(&basket)))
}

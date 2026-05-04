//! `DELETE /api/basket/:customer_id` handler.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    BasketRepository::delete_by_customer_id(&mut tx, CustomerId::from(customer_id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

//! `GET /orders/:id` handler.

use axum::Json;
use axum::extract::{Path, State};
use ordering_domain::OrderId;
use ordering_infrastructure::OrderRepository;
use uuid::Uuid;

use crate::error::Error;
use crate::response::GetOrderResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GetOrderResponse>, Error> {
    // FFI-mut exception: toasty's Connection::transaction takes &mut self.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let order = OrderRepository::get_by_id(&mut tx, OrderId::from(id)).await?;
    tx.commit().await?;
    Ok(Json(GetOrderResponse::from(&order)))
}

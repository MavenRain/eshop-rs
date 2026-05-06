//! `GET /api/orders/{id}` handler.
//!
//! Authenticated.  After loading the order, verify that
//! [`Order::user_id`](ordering_domain::Order::user_id) matches the
//! caller's JWT user-id; on mismatch we return [`Error::NotFound`]
//! rather than [`Error::Forbidden`] so the API does not leak the
//! existence of orders the caller does not own.

use axum::Json;
use axum::extract::{Path, State};
use identity_middleware::AuthenticatedPrincipal;
use ordering_domain::OrderId;
use ordering_infrastructure::OrderRepository;
use uuid::Uuid;

use crate::error::Error;
use crate::response::GetOrderResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
    Path(id): Path<Uuid>,
) -> Result<Json<GetOrderResponse>, Error> {
    let caller_user_id = principal.principal().user_id().to_string();
    // FFI-mut exception: toasty's Connection::transaction takes &mut self.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let order = OrderRepository::get_by_id(&mut tx, OrderId::from(id)).await?;
    tx.commit().await?;
    (order.user_id().as_str() == caller_user_id)
        .then(|| Json(GetOrderResponse::from(&order)))
        .ok_or_else(|| Error::NotFound {
            reason: format!("order {id} not found"),
        })
}

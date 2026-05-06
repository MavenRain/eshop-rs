//! `GET /api/orders` handler.
//!
//! Returns the orders owned by the authenticated principal.  Order
//! ownership is determined by [`Order::user_id`](ordering_domain::Order::user_id),
//! which the basket-checkout subscriber stamps from the JWT principal
//! that authored the basket.  No buyer aggregate is required: that
//! piece of upstream eShop is not yet ported.

use axum::Json;
use axum::extract::State;
use identity_middleware::AuthenticatedPrincipal;
use ordering_infrastructure::OrderRepository;

use crate::error::Error;
use crate::response::GetOrderResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
) -> Result<Json<Vec<GetOrderResponse>>, Error> {
    let user_id = principal.principal().user_id().to_string();
    // FFI-mut exception: toasty's Connection::transaction takes &mut self.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let orders = OrderRepository::list_by_user_id(&mut tx, &user_id).await?;
    tx.commit().await?;
    let body: Vec<GetOrderResponse> = orders.iter().map(GetOrderResponse::from).collect();
    Ok(Json(body))
}

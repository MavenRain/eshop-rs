//! `GET /api/basket` handler.  Returns the authenticated customer's basket.

use axum::Json;
use axum::extract::State;
use identity_middleware::AuthenticatedPrincipal;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::response::BasketResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
) -> Result<Json<BasketResponse>, Error> {
    let customer_id = CustomerId::from(principal.principal().user_id());
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let basket = BasketRepository::get_by_customer_id(&mut tx, customer_id).await?;
    tx.commit().await?;
    Ok(Json(BasketResponse::from(&basket)))
}

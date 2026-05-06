//! `DELETE /api/basket` handler.

use axum::extract::State;
use axum::http::StatusCode;
use identity_middleware::AuthenticatedPrincipal;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
) -> Result<StatusCode, Error> {
    let customer_id = CustomerId::from(principal.principal().user_id());
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    BasketRepository::delete_by_customer_id(&mut tx, customer_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

//! `PUT /api/basket` handler.
//!
//! Replace semantics: the request body holds the new full basket
//! (items only — the owner is the authenticated principal); the
//! repository deletes the prior items + customer row and inserts
//! the new pair atomically inside a single toasty transaction.

use axum::Json;
use axum::extract::State;
use identity_middleware::AuthenticatedPrincipal;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::request::UpdateBasketRequest;
use crate::response::BasketResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
    Json(request): Json<UpdateBasketRequest>,
) -> Result<Json<BasketResponse>, Error> {
    let customer_id = CustomerId::from(principal.principal().user_id());
    let basket = request.try_into_basket(customer_id)?;
    // FFI-mut exception: toasty's Connection::transaction takes `&mut self`,
    // and Transaction is then used as `&mut Executor` by the repository.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    BasketRepository::save(&mut tx, &basket).await?;
    tx.commit().await?;
    Ok(Json(BasketResponse::from(&basket)))
}

//! `PUT /api/basket/:customer_id` handler.
//!
//! Replace semantics: the request body holds the new full basket;
//! the repository deletes the prior items + customer row and inserts
//! the new pair atomically inside a single toasty transaction.

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::customer::CustomerId;
use crate::error::Error;
use crate::repository::BasketRepository;
use crate::request::UpdateBasketRequest;
use crate::response::BasketResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Json(request): Json<UpdateBasketRequest>,
) -> Result<Json<BasketResponse>, Error> {
    let basket = request.try_into_basket(CustomerId::from(customer_id))?;
    // FFI-mut exception: toasty's Connection::transaction takes `&mut self`,
    // and Transaction is then used as `&mut Executor` by the repository.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    BasketRepository::save(&mut tx, &basket).await?;
    tx.commit().await?;
    Ok(Json(BasketResponse::from(&basket)))
}

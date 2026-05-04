//! `POST /api/basket/:customer_id/checkout` handler.
//!
//! Loads the basket, validates it is non-empty, deletes it, and
//! returns the snapshot the API consumer can hand to ordering.
//!
//! Upstream eShop publishes `UserCheckoutAcceptedIntegrationEvent` at
//! this point and lets ordering consume it to mint an order.  Our
//! port defers that publish to the basket integration-events / processor
//! follow-up slice; today the response carries the full basket so an
//! HTTP-level orchestrator (or test harness) can drive the next step
//! synchronously.

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
    let id = CustomerId::from(customer_id);
    // FFI-mut exception: see `update_basket::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let basket = BasketRepository::get_by_customer_id(&mut tx, id).await?;
    if basket.is_empty() {
        Err(Error::Validation {
            reason: "cannot checkout an empty basket".to_string(),
        })
    } else {
        BasketRepository::delete_by_customer_id(&mut tx, id).await?;
        tx.commit().await?;
        Ok(Json(BasketResponse::from(&basket)))
    }
}

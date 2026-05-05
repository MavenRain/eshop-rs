//! `GET /api/webhooks/{id}` handler.

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::error::Error;
use crate::repository::WebhookSubscriptionRepository;
use crate::response::WebhookSubscriptionResponse;
use crate::state::AppState;
use crate::subscription::WebhookSubscriptionId;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebhookSubscriptionResponse>, Error> {
    // FFI-mut exception: see `register_subscription::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let subscription =
        WebhookSubscriptionRepository::get_by_id(&mut tx, WebhookSubscriptionId::from(id)).await?;
    tx.commit().await?;
    Ok(Json(WebhookSubscriptionResponse::from(&subscription)))
}

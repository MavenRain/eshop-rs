//! `DELETE /api/webhooks/{id}` handler.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::error::Error;
use crate::repository::WebhookSubscriptionRepository;
use crate::state::AppState;
use crate::subscription::WebhookSubscriptionId;

pub async fn handle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    // FFI-mut exception: see `register_subscription::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    WebhookSubscriptionRepository::delete_by_id(&mut tx, WebhookSubscriptionId::from(id)).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

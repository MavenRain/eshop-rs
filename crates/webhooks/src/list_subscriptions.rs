//! `GET /api/webhooks?grantor_id=...` handler.

use axum::Json;
use axum::extract::{Query, State};

use crate::error::Error;
use crate::repository::WebhookSubscriptionRepository;
use crate::request::ListSubscriptionsQuery;
use crate::response::WebhookSubscriptionResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Query(query): Query<ListSubscriptionsQuery>,
) -> Result<Json<Vec<WebhookSubscriptionResponse>>, Error> {
    // FFI-mut exception: see `register_subscription::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let subscriptions =
        WebhookSubscriptionRepository::list_by_grantor(&mut tx, query.grantor_id()).await?;
    tx.commit().await?;
    let response: Vec<WebhookSubscriptionResponse> = subscriptions
        .iter()
        .map(WebhookSubscriptionResponse::from)
        .collect();
    Ok(Json(response))
}

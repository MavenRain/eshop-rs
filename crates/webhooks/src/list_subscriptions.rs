//! `GET /api/webhooks` handler.  Lists subscriptions owned by the
//! authenticated principal.

use axum::Json;
use axum::extract::State;
use identity_middleware::AuthenticatedPrincipal;

use crate::error::Error;
use crate::repository::WebhookSubscriptionRepository;
use crate::response::WebhookSubscriptionResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
) -> Result<Json<Vec<WebhookSubscriptionResponse>>, Error> {
    // FFI-mut exception: see `register_subscription::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let subscriptions =
        WebhookSubscriptionRepository::list_by_grantor(&mut tx, principal.principal().user_id())
            .await?;
    tx.commit().await?;
    let response: Vec<WebhookSubscriptionResponse> = subscriptions
        .iter()
        .map(WebhookSubscriptionResponse::from)
        .collect();
    Ok(Json(response))
}

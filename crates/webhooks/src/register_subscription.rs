//! `POST /api/webhooks` handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use identity_middleware::AuthenticatedPrincipal;

use crate::error::Error;
use crate::repository::WebhookSubscriptionRepository;
use crate::request::RegisterSubscriptionRequest;
use crate::response::CreatedSubscriptionResponse;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    principal: AuthenticatedPrincipal,
    Json(request): Json<RegisterSubscriptionRequest>,
) -> Result<(StatusCode, Json<CreatedSubscriptionResponse>), Error> {
    let grantor_id = principal.principal().user_id();
    let subscription = request.try_into_subscription(grantor_id)?;
    let id = subscription.id();
    // FFI-mut exception: see `ordering-api::create_order::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    WebhookSubscriptionRepository::add(&mut tx, &subscription).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedSubscriptionResponse::new(id.into_uuid())),
    ))
}

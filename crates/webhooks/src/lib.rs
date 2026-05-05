//! Webhooks bounded context for the `eshop-rs` port of
//! [`dotnet/eShop`'s `Webhooks.API`](https://github.com/dotnet/eShop/tree/main/src/Webhooks.API).
//!
//! This sub-slice ports the subscription-management surface:
//! domain ([`WebhookSubscription`]), persistence
//! ([`WebhookSubscriptionRepository`] backed by toasty), and the
//! axum HTTP API ([`build_router`]).  The event-driven delivery
//! worker (subscribe to integration events, POST to matching
//! subscribers) lives in a separate crate and lands in a follow-on
//! sub-slice.
//!
//! # Endpoints
//!
//! - `GET    /api/webhooks?grantor_id=...`   list subscriptions for the
//!   given grantor.
//! - `GET    /api/webhooks/{id}`              fetch one.
//! - `POST   /api/webhooks`                   register a new
//!   subscription.
//! - `DELETE /api/webhooks/{id}`              drop one.
//!
//! Authentication is intentionally absent at this layer; the request
//! body / query string carry `grantor_id` directly until an identity
//! bounded context lands.

mod delete_subscription;
pub mod error;
mod get_subscription;
mod list_subscriptions;
pub mod mapper;
mod register_subscription;
mod repository;
pub mod request;
pub mod response;
pub mod row;
mod state;
pub mod strings;
pub mod subscription;
mod time;

pub use error::Error;
pub use repository::WebhookSubscriptionRepository;
pub use request::{ListSubscriptionsQuery, RegisterSubscriptionRequest};
pub use response::{CreatedSubscriptionResponse, WebhookSubscriptionResponse};
pub use state::AppState;
pub use strings::{WebhookToken, WebhookType, WebhookUrl};
pub use subscription::{WebhookSubscription, WebhookSubscriptionId};

use axum::Router;
use axum::routing::get;

/// Build the axum [`Router`] for the Webhooks API, with `state` injected.
///
/// Routes:
///
/// - `GET    /api/webhooks?grantor_id=...`
/// - `POST   /api/webhooks`
/// - `GET    /api/webhooks/{id}`
/// - `DELETE /api/webhooks/{id}`
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/webhooks",
            get(list_subscriptions::handle).post(register_subscription::handle),
        )
        .route(
            "/api/webhooks/{id}",
            get(get_subscription::handle).delete(delete_subscription::handle),
        )
        .with_state(state)
}

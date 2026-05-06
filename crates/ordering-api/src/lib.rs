//! HTTP API for the Ordering bounded context.
//!
//! Exposes a [`build_router`] that wires the upstream-aligned endpoints,
//! all under the `/api/orders` prefix:
//!
//! - `GET    /api/orders`        list orders owned by the authenticated
//!   caller.
//! - `GET    /api/orders/{id}`   fetch one order owned by the
//!   authenticated caller.
//! - `POST   /api/orders`        create an order (admin/test).
//! - `PUT    /api/orders/cancel` cancel an order (admin/test).
//! - `PUT    /api/orders/ship`   ship an order (admin/test).
//!
//! The two read endpoints are JWT-gated; the three write endpoints are
//! left ungated for now since the user-facing checkout path mints orders
//! through the basket-checkout saga, not through HTTP.
//!
//! Each command handler runs inside a toasty [`Transaction`](toasty::Transaction):
//! the aggregate change and the matching outbox writes commit atomically.
//! A separate worker drains the outbox and publishes through `event-bus`.

mod cancel_order;
mod create_order;
mod error;
mod get_order;
mod list_orders;
mod outbox;
mod request;
mod response;
mod ship_order;
mod state;

pub use error::Error;
pub use request::{
    CancelOrderRequest, CreateOrderItemRequest, CreateOrderRequest, ShipOrderRequest,
};
pub use response::{CreateOrderResponse, GetOrderResponse, OrderItemResponse};
pub use state::AppState;

use axum::Router;
use axum::routing::{get, put};

/// Build the axum [`Router`] for the Ordering API, with `state` injected.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/orders",
            get(list_orders::handle).post(create_order::handle),
        )
        .route("/api/orders/cancel", put(cancel_order::handle))
        .route("/api/orders/ship", put(ship_order::handle))
        .route("/api/orders/{id}", get(get_order::handle))
        .with_state(state)
}

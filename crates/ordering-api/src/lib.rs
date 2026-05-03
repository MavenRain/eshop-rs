//! HTTP API for the Ordering bounded context.
//!
//! Exposes a [`build_router`] that wires the four endpoints used by upstream:
//!
//! - `POST /orders` create an order
//! - `PUT /orders/cancel` cancel an order
//! - `PUT /orders/ship` ship an order
//! - `GET /orders/:id` fetch an order by id
//!
//! Each command handler runs inside a toasty [`Transaction`](toasty::Transaction):
//! the aggregate change and the matching outbox writes commit atomically.
//! A separate worker drains the outbox and publishes through `event-bus`;
//! that worker is step 9 of the punchlist (`OrderProcessor`).

mod cancel_order;
mod create_order;
mod error;
mod get_order;
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
use axum::routing::{get, post, put};

/// Build the axum [`Router`] for the Ordering API, with `state` injected.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/orders", post(create_order::handle))
        .route("/orders/cancel", put(cancel_order::handle))
        .route("/orders/ship", put(ship_order::handle))
        .route("/orders/{id}", get(get_order::handle))
        .with_state(state)
}

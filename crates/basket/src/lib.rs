//! Basket bounded context for the `eshop-rs` port of
//! [`dotnet/eShop`'s `Basket.API`](https://github.com/dotnet/eShop/tree/main/src/Basket.API).
//!
//! This crate ports the domain layer ([`CustomerBasket`],
//! [`BasketItem`]), the persistence layer ([`BasketRepository`]
//! backed by toasty), and the axum HTTP API mounted by
//! [`build_router`].
//!
//! # Endpoints
//!
//! - `GET    /api/basket/{customer_id}`       fetch the basket
//! - `PUT    /api/basket/{customer_id}`       replace the basket
//! - `DELETE /api/basket/{customer_id}`       drop the basket
//! - `POST   /api/basket/{customer_id}/checkout`
//!   load + drop the basket and return its snapshot
//!
//! Upstream uses gRPC and Redis; the Rust port lands on REST/JSON
//! and toasty for parity with the other bounded contexts.  Upstream
//! emits `UserCheckoutAcceptedIntegrationEvent` at the checkout site;
//! that publish lands alongside the basket integration-events /
//! processor follow-up slice.
//!
//! Toasty model registration: applications mount the row models via
//! `toasty::models!(...)` at the `Db::builder` site; the
//! [`CustomerBasketRow`](crate::row::CustomerBasketRow) and
//! [`BasketItemRow`](crate::row::BasketItemRow) types are picked up
//! through the same inventory mechanism the catalog and ordering
//! crates already use.

pub mod basket;
pub mod basket_item;
mod checkout_basket;
pub mod customer;
mod delete_basket;
pub mod error;
pub mod event;
mod get_basket;
pub mod integration_event_log;
pub mod mapper;
pub mod money;
mod outbox;
pub mod product;
mod repository;
pub mod request;
pub mod response;
pub mod row;
mod state;
mod strings;
mod time;
mod update_basket;

pub use basket::CustomerBasket;
pub use basket_item::{BasketItem, BasketItemId};
pub use customer::CustomerId;
pub use error::Error;
pub use event::{CheckoutAcceptedEvent, DomainEvent};
pub use integration_event_log::{
    EventLog, EventState, IntegrationEventLogService, PendingEventLog,
};
pub use money::{Price, Quantity};
pub use product::ProductId;
pub use repository::BasketRepository;
pub use request::{UpdateBasketItemRequest, UpdateBasketRequest};
pub use response::{BasketItemResponse, BasketResponse};
pub use state::AppState;
pub use strings::{PictureUrl, ProductName};

use axum::Router;
use axum::routing::get;

/// Build the axum [`Router`] for the Basket API, with `state` injected.
///
/// Customer identity is read from the authenticated principal
/// (Authorization: Bearer …), not the URL.  Routes:
///
/// - `GET    /api/basket`
/// - `PUT    /api/basket`
/// - `DELETE /api/basket`
/// - `POST   /api/basket/checkout`
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/basket",
            get(get_basket::handle)
                .put(update_basket::handle)
                .delete(delete_basket::handle),
        )
        .route(
            "/api/basket/checkout",
            axum::routing::post(checkout_basket::handle),
        )
        .with_state(state)
}

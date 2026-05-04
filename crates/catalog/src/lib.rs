//! Catalog bounded context for the `eshop-rs` port of
//! [`dotnet/eShop`'s `Catalog.API`](https://github.com/dotnet/eShop/tree/main/src/Catalog.API).
//!
//! The crate ports the domain layer (aggregates, value objects, the
//! [`DomainEvent`] sum type with [`ProductPriceChangedEvent`]), the
//! persistence layer ([`CatalogItemRepository`],
//! [`CatalogBrandRepository`], [`CatalogKindRepository`] backed by
//! toasty), and the axum HTTP API mounted by [`build_router`].
//!
//! Each command handler runs inside a toasty
//! [`Transaction`](toasty::Transaction): the aggregate change and any
//! matching outbox writes commit atomically.  A separate worker
//! drains [`CatalogIntegrationEventLogRow`](crate::row::CatalogIntegrationEventLogRow)
//! rows and publishes through `event-bus`; that worker arrives with
//! the eShop.AppHost replacement, mirroring `OrderProcessor`.
//!
//! Upstream's AI / pgvector semantic search and picture-file streaming
//! are explicitly out of scope for v1.
//!
//! Toasty model registration: applications mount the row models via
//! `toasty::models!(...)` at the `Db::builder` site; the
//! [`CatalogItemRow`](crate::row::CatalogItemRow) /
//! [`CatalogBrandRow`](crate::row::CatalogBrandRow) /
//! [`CatalogKindRow`](crate::row::CatalogKindRow) /
//! [`CatalogIntegrationEventLogRow`](crate::row::CatalogIntegrationEventLogRow)
//! types are picked up through the same inventory mechanism
//! `ordering-infrastructure` already uses.

pub mod brand;
mod create_brand;
mod create_item;
mod create_kind;
mod delete_item;
pub mod error;
pub mod event;
mod get_item;
pub mod integration_event_log;
pub mod item;
pub mod kind;
mod list_brands;
mod list_items;
mod list_kinds;
mod mapper;
pub mod money;
mod outbox;
mod repository;
pub mod request;
pub mod response;
pub mod row;
mod state;
pub mod stock;
pub mod strings;
mod time;
mod update_item;

pub use brand::{CatalogBrand, CatalogBrandId};
pub use error::Error;
pub use event::{DomainEvent, ProductPriceChangedEvent};
pub use integration_event_log::{
    EventLog, EventState, IntegrationEventLogService, PendingEventLog,
};
pub use item::{CatalogItem, CatalogItemId};
pub use kind::{CatalogKind, CatalogKindId};
pub use money::Price;
pub use repository::{CatalogBrandRepository, CatalogItemRepository, CatalogKindRepository};
pub use request::{
    CreateBrandRequest, CreateItemRequest, CreateKindRequest, PaginationQuery, UpdateItemRequest,
};
pub use response::{
    CatalogBrandResponse, CatalogItemResponse, CatalogKindResponse, CreatedResponse,
    PaginatedItemsResponse,
};
pub use state::AppState;
pub use stock::{MaxStockThreshold, RestockThreshold, Stock, Units};
pub use strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

use axum::Router;
use axum::routing::get;

/// Build the axum [`Router`] for the Catalog API, with `state` injected.
///
/// Routes mirror upstream `Catalog.API`'s minimal-API surface:
///
/// - `GET    /api/catalog/items` — paginated list
/// - `GET    /api/catalog/items/{id}`
/// - `POST   /api/catalog/items`
/// - `PUT    /api/catalog/items/{id}`
/// - `DELETE /api/catalog/items/{id}`
/// - `GET    /api/catalog/brands`
/// - `POST   /api/catalog/brands`
/// - `GET    /api/catalog/kinds`
/// - `POST   /api/catalog/kinds`
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/catalog/items",
            get(list_items::handle).post(create_item::handle),
        )
        .route(
            "/api/catalog/items/{id}",
            get(get_item::handle)
                .put(update_item::handle)
                .delete(delete_item::handle),
        )
        .route(
            "/api/catalog/brands",
            get(list_brands::handle).post(create_brand::handle),
        )
        .route(
            "/api/catalog/kinds",
            get(list_kinds::handle).post(create_kind::handle),
        )
        .with_state(state)
}

//! Catalog bounded context for the `eshop-rs` port of
//! [`dotnet/eShop`'s `Catalog.API`](https://github.com/dotnet/eShop/tree/main/src/Catalog.API).
//!
//! This crate ports the domain layer (aggregates, value objects, the
//! [`DomainEvent`] sum type with [`ProductPriceChangedEvent`]) and the
//! persistence layer ([`CatalogItemRepository`],
//! [`CatalogBrandRepository`], [`CatalogKindRepository`] backed by
//! toasty).  The axum HTTP API lands in a follow-up commit.
//!
//! Upstream's AI / pgvector semantic search and picture-file streaming
//! are explicitly out of scope for v1.
//!
//! Toasty model registration: applications mount the row models via
//! `toasty::models!(...)` at the `Db::builder` site; the
//! [`CatalogItemRow`](crate::row::CatalogItemRow) /
//! [`CatalogBrandRow`](crate::row::CatalogBrandRow) /
//! [`CatalogKindRow`](crate::row::CatalogKindRow) types are picked up
//! through the same inventory mechanism `ordering-infrastructure`
//! already uses.

pub mod brand;
pub mod error;
pub mod event;
pub mod item;
pub mod kind;
mod mapper;
pub mod money;
mod repository;
mod row;
pub mod stock;
pub mod strings;

pub use brand::{CatalogBrand, CatalogBrandId};
pub use error::Error;
pub use event::{DomainEvent, ProductPriceChangedEvent};
pub use item::{CatalogItem, CatalogItemId};
pub use kind::{CatalogKind, CatalogKindId};
pub use money::Price;
pub use repository::{CatalogBrandRepository, CatalogItemRepository, CatalogKindRepository};
pub use stock::{MaxStockThreshold, RestockThreshold, Stock, Units};
pub use strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

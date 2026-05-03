//! Catalog bounded context: domain types for the `eshop-rs` port of
//! [`dotnet/eShop`'s `Catalog.API`](https://github.com/dotnet/eShop/tree/main/src/Catalog.API).
//!
//! This first slice ports the domain layer only: aggregates
//! ([`CatalogItem`], [`CatalogBrand`], [`CatalogKind`]), value objects
//! ([`Price`], [`Stock`], [`RestockThreshold`], [`MaxStockThreshold`],
//! [`Units`], plus the non-empty string newtypes), and the
//! [`DomainEvent`] sum type with the single
//! [`ProductPriceChangedEvent`] payload published by upstream when a
//! catalog item's price changes.
//!
//! Persistence (toasty rows, repositories) and transport (axum HTTP API)
//! land in follow-up commits.  Upstream's AI / pgvector semantic search
//! and picture-file streaming are explicitly out of scope for v1.

pub mod brand;
pub mod error;
pub mod event;
pub mod item;
pub mod kind;
pub mod money;
pub mod stock;
pub mod strings;

pub use brand::{CatalogBrand, CatalogBrandId};
pub use error::Error;
pub use event::{DomainEvent, ProductPriceChangedEvent};
pub use item::{CatalogItem, CatalogItemId};
pub use kind::{CatalogKind, CatalogKindId};
pub use money::Price;
pub use stock::{MaxStockThreshold, RestockThreshold, Stock, Units};
pub use strings::{BrandName, ItemDescription, ItemName, KindName, PictureFileName};

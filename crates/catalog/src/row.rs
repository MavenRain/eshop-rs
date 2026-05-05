//! Toasty `Model` rows for the Catalog bounded context.
//!
//! These types are crate-private mirrors of the database schema.  They
//! are never exposed in this crate's public API; mappers in
//! [`crate::mapper`] translate between rows and the domain aggregates
//! ([`CatalogItem`](crate::item::CatalogItem),
//! [`CatalogBrand`](crate::brand::CatalogBrand),
//! [`CatalogKind`](crate::kind::CatalogKind)).
//!
//! Stock columns are stored as `i64` rather than `u32` to match the
//! `units: i64` precedent in `ordering-infrastructure`'s
//! `OrderItemRow`; the mapper layer enforces the `u32` round-trip.

use jiff::Timestamp;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, toasty::Model)]
pub struct CatalogItemRow {
    /// Catalog product id.  Width: `i32`, matching upstream eShop and
    /// the `OrderStockItem`/`ProductId` shape used by ordering and
    /// basket integration events.  Caller-supplied (the request body
    /// carries the value); auto-incrementing IDs at the database
    /// layer are a future toasty enhancement.
    #[key]
    pub id: i32,

    pub name: String,

    pub description: Option<String>,

    pub price: Decimal,

    pub picture_file_name: Option<String>,

    #[index]
    pub brand_id: Uuid,

    #[index]
    pub kind_id: Uuid,

    pub available_stock: i64,

    pub restock_threshold: i64,

    pub max_stock_threshold: i64,

    pub on_reorder: bool,
}

#[derive(Debug, toasty::Model)]
pub struct CatalogBrandRow {
    #[key]
    #[auto]
    pub id: Uuid,

    pub name: String,
}

#[derive(Debug, toasty::Model)]
pub struct CatalogKindRow {
    #[key]
    #[auto]
    pub id: Uuid,

    pub name: String,
}

/// Catalog-side outbox row.  Distinct from
/// `ordering-infrastructure::IntegrationEventLogRow` so toasty model
/// inventory registers two separate tables; mirrors upstream eShop's
/// per-service outbox table convention.
#[derive(Debug, toasty::Model)]
pub struct CatalogIntegrationEventLogRow {
    #[key]
    pub event_id: Uuid,

    pub event_type_name: String,

    pub state: String,

    pub times_sent: i64,

    pub creation_time: Timestamp,

    pub content: String,

    #[index]
    pub transaction_id: Uuid,
}

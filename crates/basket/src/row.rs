//! Toasty rows for the Basket bounded context.
//!
//! Two rows: [`CustomerBasketRow`] keyed by customer id, and
//! [`BasketItemRow`] with an indexed FK column back to the
//! customer.  No `belongs_to` / `has_many` wiring: the catalog crate
//! sets the precedent of keeping aggregate boundaries explicit at
//! the schema layer too.
//!
//! `quantity` is stored as `i64` because that is the column type
//! `toasty::Model` expects; the mapper enforces the `u32`
//! round-trip and surfaces [`Error::NumericRange`](crate::Error::NumericRange)
//! on out-of-range persisted values.

use jiff::Timestamp;
use rust_decimal::Decimal;
use uuid::Uuid;

/// One row per customer basket.
#[derive(Debug, toasty::Model)]
pub struct CustomerBasketRow {
    /// Customer id (also the primary key).
    #[key]
    pub id: Uuid,
}

/// One row per line item.
#[derive(Debug, toasty::Model)]
pub struct BasketItemRow {
    /// Item identifier.
    #[key]
    pub id: Uuid,

    /// Customer basket this item belongs to.
    #[index]
    pub customer_basket_id: Uuid,

    /// Catalog product id (snapshot).
    pub product_id: i32,

    /// Snapshotted product name.
    pub product_name: String,

    /// Current unit price.
    pub unit_price: Decimal,

    /// Previous unit price when the catalog has changed.
    pub old_unit_price: Option<Decimal>,

    /// Number of units ordered.  Stored as `i64`; the mapper enforces
    /// the strictly-positive `u32` round-trip.
    pub quantity: i64,

    /// Optional picture url snapshot for the basket UI.
    pub picture_url: Option<String>,
}

/// Basket-side outbox row.  Distinct from
/// `ordering-infrastructure::IntegrationEventLogRow` and
/// `catalog::CatalogIntegrationEventLogRow` so toasty model
/// inventory registers three separate tables; mirrors upstream
/// eShop's per-service outbox table convention.
#[derive(Debug, toasty::Model)]
pub struct BasketIntegrationEventLogRow {
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

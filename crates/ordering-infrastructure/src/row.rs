//! Toasty `Model` rows for the Ordering bounded context.
//!
//! These types are crate-private mirrors of the database schema.  They are
//! never exposed in this crate's public API; mappers in [`crate::mapper`]
//! translate between rows and the [`ordering_domain`] aggregates.

use jiff::Timestamp;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, toasty::Model)]
pub struct OrderRow {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    pub user_id: String,

    pub order_date: Timestamp,

    pub order_status: String,

    pub description: String,

    pub buyer_id: Option<Uuid>,

    pub payment_method_id: Option<Uuid>,

    pub street: String,

    pub city: String,

    pub state: String,

    pub country: String,

    pub zip_code: String,

    #[has_many]
    pub items: toasty::HasMany<OrderItemRow>,
}

#[derive(Debug, toasty::Model)]
pub struct OrderItemRow {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    pub order_id: Uuid,

    #[belongs_to(key = order_id, references = id)]
    pub order_row: toasty::BelongsTo<OrderRow>,

    pub product_id: i64,

    pub product_name: String,

    pub picture_url: String,

    pub unit_price: Decimal,

    pub discount: Decimal,

    pub units: i64,
}

#[derive(Debug, toasty::Model)]
pub struct BuyerRow {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub identity_guid: String,

    pub name: String,

    #[has_many]
    pub payment_methods: toasty::HasMany<PaymentMethodRow>,
}

#[derive(Debug, toasty::Model)]
pub struct PaymentMethodRow {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    pub buyer_id: Uuid,

    #[belongs_to(key = buyer_id, references = id)]
    pub buyer_row: toasty::BelongsTo<BuyerRow>,

    pub alias: String,

    pub card_number: String,

    pub security_number: String,

    pub card_holder_name: String,

    pub expiration: Timestamp,

    pub card_type_id: i64,
}

#[derive(Debug, toasty::Model)]
pub struct IntegrationEventLogRow {
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

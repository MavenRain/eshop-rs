//! Bidirectional mappers between domain aggregates and database rows.
//!
//! Domain types preserve their invariants (private fields, validated
//! constructors); rows are pure bags of columns.  These functions are the
//! single boundary where the two representations meet.

use ordering_domain::{
    Address, Buyer, BuyerId, CardAlias, CardExpiration, CardHolderName, CardNumber, CardType, City,
    Country, Description, Discount, IdentityGuid, Money, Order, OrderDate, OrderId, OrderItem,
    OrderItemId, OrderStatus, PaymentMethod, PaymentMethodId, PictureUrl, ProductId, ProductName,
    SecurityNumber, State, Street, UnitPrice, Units, UserName, ZipCode,
};

use crate::error::Error;
use crate::row::{BuyerRow, OrderItemRow, OrderRow, PaymentMethodRow};
use crate::time::{chrono_to_jiff, jiff_to_chrono};

/// Project an [`Order`] aggregate into the database-side row plus its
/// child item rows.
pub fn order_to_rows(order: &Order) -> Result<(NewOrderRow, Vec<NewOrderItemRow>), Error> {
    let order_id = order.id().into_uuid();
    let row = NewOrderRow {
        id: order_id,
        order_date: chrono_to_jiff(order.order_date().into_inner())?,
        order_status: order.order_status().to_string(),
        description: order.description().as_str().to_string(),
        buyer_id: order.buyer_id().map(BuyerId::into_uuid),
        payment_method_id: order.payment_id().map(PaymentMethodId::into_uuid),
        street: order.address().street().as_str().to_string(),
        city: order.address().city().as_str().to_string(),
        state: order.address().state().as_str().to_string(),
        country: order.address().country().as_str().to_string(),
        zip_code: order.address().zip_code().as_str().to_string(),
    };
    let items = order
        .order_items()
        .iter()
        .map(|item| {
            let units_i64 = i64::from(item.units().get());
            let product_id_i64 = i64::from(i32::from(item.product_id()));
            Ok::<NewOrderItemRow, Error>(NewOrderItemRow {
                id: item.id().into_uuid(),
                order_id,
                product_id: product_id_i64,
                product_name: item.product_name().as_str().to_string(),
                picture_url: item.picture_url().as_str().to_string(),
                unit_price: item.unit_price().money().into(),
                discount: item.discount().money().into(),
                units: units_i64,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((row, items))
}

/// Rehydrate an [`Order`] from a row plus its loaded item rows.
pub fn rows_to_order(row: &OrderRow, items: &[OrderItemRow]) -> Result<Order, Error> {
    let address = Address::new(
        Street::try_from(row.street.clone())?,
        City::try_from(row.city.clone())?,
        State::try_from(row.state.clone())?,
        Country::try_from(row.country.clone())?,
        ZipCode::try_from(row.zip_code.clone())?,
    );
    let order_status = OrderStatus::parse_display(&row.order_status).ok_or_else(|| {
        Error::InvalidPersistedValue {
            context: "orders.order_status".to_string(),
            value: row.order_status.clone(),
        }
    })?;
    let order_date = OrderDate::from(jiff_to_chrono(row.order_date)?);
    let order_items: Vec<OrderItem> = items
        .iter()
        .map(row_to_order_item)
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Order::rehydrate(
        OrderId::from(row.id),
        order_date,
        address,
        row.buyer_id.map(BuyerId::from),
        order_status,
        Description::from(row.description.clone()),
        order_items,
        row.payment_method_id.map(PaymentMethodId::from),
    ))
}

fn row_to_order_item(row: &OrderItemRow) -> Result<OrderItem, Error> {
    let product_id_i32 = i32::try_from(row.product_id).map_err(|_| Error::NumericRange {
        context: "order_items.product_id".to_string(),
    })?;
    let units_u32 = u32::try_from(row.units).map_err(|_| Error::NumericRange {
        context: "order_items.units".to_string(),
    })?;
    Ok(OrderItem::new(
        OrderItemId::from(row.id),
        ProductId::from(product_id_i32),
        ProductName::try_from(row.product_name.clone())?,
        PictureUrl::try_from(row.picture_url.clone())?,
        UnitPrice::from(Money::from(row.unit_price)),
        Discount::new(Money::from(row.discount))?,
        Units::new(units_u32)?,
    )?)
}

/// Project a [`Buyer`] aggregate into row form (buyer + payment methods).
pub fn buyer_to_rows(buyer: &Buyer) -> Result<(NewBuyerRow, Vec<NewPaymentMethodRow>), Error> {
    let buyer_id = buyer.id().into_uuid();
    let buyer_row = NewBuyerRow {
        id: buyer_id,
        identity_guid: buyer.identity_guid().as_str().to_string(),
        name: buyer.name().as_str().to_string(),
    };
    let payment_methods = buyer
        .payment_methods()
        .iter()
        .map(|pm| {
            Ok::<NewPaymentMethodRow, Error>(NewPaymentMethodRow {
                id: pm.id().into_uuid(),
                buyer_id,
                alias: pm.alias().as_str().to_string(),
                card_number: pm.card_number().as_str().to_string(),
                security_number: pm.security_number().as_str().to_string(),
                card_holder_name: pm.card_holder_name().as_str().to_string(),
                expiration: chrono_to_jiff(pm.expiration().into_inner())?,
                card_type_id: i64::from(pm.card_type().id()),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok((buyer_row, payment_methods))
}

/// Rehydrate a [`Buyer`] from a row plus its loaded payment-method rows.
pub fn rows_to_buyer(row: &BuyerRow, payments: &[PaymentMethodRow]) -> Result<Buyer, Error> {
    let identity_guid = IdentityGuid::try_from(row.identity_guid.clone())?;
    let name = UserName::try_from(row.name.clone())?;
    let buyer = Buyer::new(BuyerId::from(row.id), identity_guid, name);
    payments.iter().try_fold(buyer, |acc, pm_row| {
        let pm = row_to_payment_method(pm_row)?;
        Ok::<Buyer, Error>(acc.with_payment_method(pm))
    })
}

fn row_to_payment_method(row: &PaymentMethodRow) -> Result<PaymentMethod, Error> {
    let card_type_i32 = i32::try_from(row.card_type_id).map_err(|_| Error::NumericRange {
        context: "payment_methods.card_type_id".to_string(),
    })?;
    let card_type = CardType::try_from(card_type_i32)?;
    let expiration_chrono = jiff_to_chrono(row.expiration)?;
    // Bypass the future-only check: persisted values may already be past.
    let expiration = CardExpiration::new(
        expiration_chrono,
        expiration_chrono - chrono::Duration::seconds(1),
    )?;
    Ok(PaymentMethod::new(
        PaymentMethodId::from(row.id),
        CardAlias::try_from(row.alias.clone())?,
        CardNumber::try_from(row.card_number.clone())?,
        SecurityNumber::try_from(row.security_number.clone())?,
        CardHolderName::try_from(row.card_holder_name.clone())?,
        expiration,
        card_type,
    ))
}

/// Plain bag-of-columns representation of [`Order`] for insert.  Avoids
/// constructing the toasty row directly so callers can vary how they
/// flush (single insert vs. transaction batch).
#[derive(Debug, Clone)]
pub struct NewOrderRow {
    pub id: uuid::Uuid,
    pub order_date: jiff::Timestamp,
    pub order_status: String,
    pub description: String,
    pub buyer_id: Option<uuid::Uuid>,
    pub payment_method_id: Option<uuid::Uuid>,
    pub street: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip_code: String,
}

#[derive(Debug, Clone)]
pub struct NewOrderItemRow {
    pub id: uuid::Uuid,
    pub order_id: uuid::Uuid,
    pub product_id: i64,
    pub product_name: String,
    pub picture_url: String,
    pub unit_price: rust_decimal::Decimal,
    pub discount: rust_decimal::Decimal,
    pub units: i64,
}

#[derive(Debug, Clone)]
pub struct NewBuyerRow {
    pub id: uuid::Uuid,
    pub identity_guid: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NewPaymentMethodRow {
    pub id: uuid::Uuid,
    pub buyer_id: uuid::Uuid,
    pub alias: String,
    pub card_number: String,
    pub security_number: String,
    pub card_holder_name: String,
    pub expiration: jiff::Timestamp,
    pub card_type_id: i64,
}

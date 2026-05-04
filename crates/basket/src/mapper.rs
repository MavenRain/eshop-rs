//! Bidirectional mappers between basket aggregates and database rows.

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::basket::CustomerBasket;
use crate::basket_item::{BasketItem, BasketItemId};
use crate::customer::CustomerId;
use crate::error::Error;
use crate::money::{Price, Quantity};
use crate::product::ProductId;
use crate::row::{BasketItemRow, CustomerBasketRow};
use crate::strings::{PictureUrl, ProductName};

/// Project a [`CustomerBasket`] into the row pair ready for insert.
///
/// Returns `(customer_row, item_rows)` where `item_rows` are already
/// FK-linked to the customer.
#[must_use]
pub fn basket_to_rows(basket: &CustomerBasket) -> (NewCustomerBasketRow, Vec<NewBasketItemRow>) {
    let customer_id = basket.customer_id().into_uuid();
    let customer_row = NewCustomerBasketRow { id: customer_id };
    let item_rows = basket
        .items()
        .iter()
        .map(|item| item_to_row(customer_id, item))
        .collect();
    (customer_row, item_rows)
}

/// Project a single [`BasketItem`] into a row, FK-linked to its
/// owning customer.
#[must_use]
pub fn item_to_row(customer_id: Uuid, item: &BasketItem) -> NewBasketItemRow {
    NewBasketItemRow {
        id: item.id().into_uuid(),
        customer_basket_id: customer_id,
        product_id: item.product_id().into(),
        product_name: item.product_name().as_str().to_string(),
        unit_price: item.unit_price().into_decimal(),
        old_unit_price: item.old_unit_price().map(Price::into_decimal),
        quantity: i64::from(item.quantity().get()),
        picture_url: item.picture_url().map(|p| p.as_str().to_string()),
    }
}

/// Rehydrate a [`CustomerBasket`] from its row + items.
///
/// # Errors
/// - [`Error::NumericRange`] if any `quantity` is outside `u32`.
/// - [`Error::ZeroQuantity`] if any persisted `quantity` is zero.
/// - [`Error::EmptyString`] / [`Error::NegativePrice`] if a persisted
///   value violates a domain invariant.
pub fn rows_to_basket(
    customer_row: &CustomerBasketRow,
    item_rows: &[BasketItemRow],
) -> Result<CustomerBasket, Error> {
    let customer_id = CustomerId::from(customer_row.id);
    let items: Vec<BasketItem> = item_rows
        .iter()
        .map(row_to_item)
        .collect::<Result<_, _>>()?;
    Ok(CustomerBasket::new(customer_id, items))
}

/// Rehydrate a single [`BasketItem`] from a loaded row.
///
/// # Errors
/// See [`rows_to_basket`].
pub fn row_to_item(row: &BasketItemRow) -> Result<BasketItem, Error> {
    let quantity_u32 = u32::try_from(row.quantity).map_err(|_| Error::NumericRange {
        context: "basket_items.quantity".to_string(),
    })?;
    let product_name = ProductName::try_from(row.product_name.clone())?;
    let picture_url = row
        .picture_url
        .as_ref()
        .map(|s| PictureUrl::try_from(s.clone()))
        .transpose()?;
    let old_unit_price = row.old_unit_price.map(Price::new).transpose()?;
    Ok(BasketItem::new(
        BasketItemId::from(row.id),
        ProductId::from(row.product_id),
        product_name,
        Price::new(row.unit_price)?,
        old_unit_price,
        Quantity::new(quantity_u32)?,
        picture_url,
    ))
}

/// Insert/replace helper for [`CustomerBasketRow`].
#[derive(Debug, Clone, Copy)]
pub struct NewCustomerBasketRow {
    pub id: Uuid,
}

/// Insert/replace helper for [`BasketItemRow`].
#[derive(Debug, Clone)]
pub struct NewBasketItemRow {
    pub id: Uuid,
    pub customer_basket_id: Uuid,
    pub product_id: i32,
    pub product_name: String,
    pub unit_price: Decimal,
    pub old_unit_price: Option<Decimal>,
    pub quantity: i64,
    pub picture_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_item() -> Result<BasketItem, Error> {
        Ok(BasketItem::new(
            BasketItemId::new(),
            ProductId::from(42),
            ProductName::try_from(".NET Bot Hoodie")?,
            Price::new(Decimal::from(20))?,
            Some(Price::new(Decimal::from(25))?),
            Quantity::new(2)?,
            Some(PictureUrl::try_from("hoodie.png")?),
        ))
    }

    fn sample_basket() -> Result<CustomerBasket, Error> {
        Ok(CustomerBasket::new(CustomerId::new(), vec![sample_item()?]))
    }

    #[test]
    fn item_round_trip_via_row() -> Result<(), Error> {
        let item = sample_item()?;
        let customer_id = Uuid::new_v4();
        let helper = item_to_row(customer_id, &item);
        let row = BasketItemRow {
            id: helper.id,
            customer_basket_id: helper.customer_basket_id,
            product_id: helper.product_id,
            product_name: helper.product_name,
            unit_price: helper.unit_price,
            old_unit_price: helper.old_unit_price,
            quantity: helper.quantity,
            picture_url: helper.picture_url,
        };
        let restored = row_to_item(&row)?;
        check(restored.id() == item.id(), || "id".to_string())?;
        check(restored.product_id() == item.product_id(), || {
            "product_id".to_string()
        })?;
        check(restored.product_name() == item.product_name(), || {
            "product_name".to_string()
        })?;
        check(restored.unit_price() == item.unit_price(), || {
            "unit_price".to_string()
        })?;
        check(restored.old_unit_price() == item.old_unit_price(), || {
            "old_unit_price".to_string()
        })?;
        check(restored.quantity() == item.quantity(), || {
            "quantity".to_string()
        })?;
        check(restored.picture_url() == item.picture_url(), || {
            "picture_url".to_string()
        })
    }

    #[test]
    fn row_to_item_rejects_zero_quantity() -> Result<(), Error> {
        let row = BasketItemRow {
            id: Uuid::new_v4(),
            customer_basket_id: Uuid::new_v4(),
            product_id: 1,
            product_name: "x".to_string(),
            unit_price: Decimal::ONE,
            old_unit_price: None,
            quantity: 0,
            picture_url: None,
        };
        let outcome = row_to_item(&row);
        check(matches!(outcome, Err(Error::ZeroQuantity)), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn row_to_item_rejects_negative_quantity() -> Result<(), Error> {
        let row = BasketItemRow {
            id: Uuid::new_v4(),
            customer_basket_id: Uuid::new_v4(),
            product_id: 1,
            product_name: "x".to_string(),
            unit_price: Decimal::ONE,
            old_unit_price: None,
            quantity: -3,
            picture_url: None,
        };
        let outcome = row_to_item(&row);
        check(matches!(outcome, Err(Error::NumericRange { .. })), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn basket_round_trip_via_rows() -> Result<(), Error> {
        let basket = sample_basket()?;
        let (helper_customer, helper_items) = basket_to_rows(&basket);
        let customer_row = CustomerBasketRow {
            id: helper_customer.id,
        };
        let item_rows: Vec<BasketItemRow> = helper_items
            .into_iter()
            .map(|h| BasketItemRow {
                id: h.id,
                customer_basket_id: h.customer_basket_id,
                product_id: h.product_id,
                product_name: h.product_name,
                unit_price: h.unit_price,
                old_unit_price: h.old_unit_price,
                quantity: h.quantity,
                picture_url: h.picture_url,
            })
            .collect();
        let restored = rows_to_basket(&customer_row, &item_rows)?;
        check(restored.customer_id() == basket.customer_id(), || {
            "customer_id".to_string()
        })?;
        check(restored.item_count() == basket.item_count(), || {
            "item_count".to_string()
        })
    }
}

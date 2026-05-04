//! Request DTOs deserialized from JSON bodies.
//!
//! Fields are private; serde derives expand in this module and have
//! access to them.  Public projection methods convert each request
//! into the corresponding domain type, validating along the way.

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::basket::CustomerBasket;
use crate::basket_item::{BasketItem, BasketItemId};
use crate::customer::CustomerId;
use crate::error::Error;
use crate::money::{Price, Quantity};
use crate::product::ProductId;
use crate::strings::{PictureUrl, ProductName};

/// `PUT /api/basket/{customer_id}` body.
///
/// Replace semantics: the basket is wholly replaced with this list of
/// items.  Per-item `id` is optional; when absent we mint a fresh
/// [`BasketItemId`].
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBasketRequest {
    items: Vec<UpdateBasketItemRequest>,
}

impl UpdateBasketRequest {
    /// Project this request and the path-bound customer id into a
    /// [`CustomerBasket`] aggregate.
    ///
    /// # Errors
    /// Propagates the per-item validation errors from
    /// [`UpdateBasketItemRequest::try_into_item`].
    pub fn try_into_basket(self, customer_id: CustomerId) -> Result<CustomerBasket, Error> {
        let items: Vec<BasketItem> = self
            .items
            .into_iter()
            .map(UpdateBasketItemRequest::try_into_item)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CustomerBasket::new(customer_id, items))
    }
}

/// One element of an [`UpdateBasketRequest::items`] list.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBasketItemRequest {
    id: Option<Uuid>,
    product_id: i32,
    product_name: String,
    unit_price: Decimal,
    old_unit_price: Option<Decimal>,
    quantity: u32,
    picture_url: Option<String>,
}

impl UpdateBasketItemRequest {
    /// Project into a [`BasketItem`].
    ///
    /// # Errors
    /// - [`Error::EmptyString`] for blank `product_name` / `picture_url`.
    /// - [`Error::NegativePrice`] for a negative unit price.
    /// - [`Error::ZeroQuantity`] for `quantity == 0`.
    pub fn try_into_item(self) -> Result<BasketItem, Error> {
        let id = self.id.map_or_else(BasketItemId::new, BasketItemId::from);
        let product_name = ProductName::try_from(self.product_name)?;
        let unit_price = Price::new(self.unit_price)?;
        let old_unit_price = self.old_unit_price.map(Price::new).transpose()?;
        let quantity = Quantity::new(self.quantity)?;
        let picture_url = self.picture_url.map(PictureUrl::try_from).transpose()?;
        Ok(BasketItem::new(
            id,
            ProductId::from(self.product_id),
            product_name,
            unit_price,
            old_unit_price,
            quantity,
            picture_url,
        ))
    }
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

    fn valid_item_request() -> UpdateBasketItemRequest {
        UpdateBasketItemRequest {
            id: None,
            product_id: 42,
            product_name: ".NET Bot Hoodie".to_string(),
            unit_price: Decimal::from(20),
            old_unit_price: Some(Decimal::from(25)),
            quantity: 2,
            picture_url: Some("hoodie.png".to_string()),
        }
    }

    #[test]
    fn item_request_projects() -> Result<(), Error> {
        let item = valid_item_request().try_into_item()?;
        check(item.product_id() == ProductId::from(42), || {
            "product_id".to_string()
        })?;
        check(item.product_name().as_str() == ".NET Bot Hoodie", || {
            "name".to_string()
        })?;
        check(item.quantity().get() == 2, || "quantity".to_string())
    }

    #[test]
    fn item_request_rejects_zero_quantity() -> Result<(), Error> {
        let request = UpdateBasketItemRequest {
            quantity: 0,
            ..valid_item_request()
        };
        let outcome = request.try_into_item();
        check(matches!(outcome, Err(Error::ZeroQuantity)), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn item_request_rejects_negative_unit_price() -> Result<(), Error> {
        let request = UpdateBasketItemRequest {
            unit_price: Decimal::NEGATIVE_ONE,
            ..valid_item_request()
        };
        let outcome = request.try_into_item();
        check(matches!(outcome, Err(Error::NegativePrice)), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn item_request_rejects_empty_product_name() -> Result<(), Error> {
        let request = UpdateBasketItemRequest {
            product_name: String::new(),
            ..valid_item_request()
        };
        let outcome = request.try_into_item();
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "product name"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn basket_request_projects_with_path_customer_id() -> Result<(), Error> {
        let request = UpdateBasketRequest {
            items: vec![valid_item_request()],
        };
        let customer_id = CustomerId::new();
        let basket = request.try_into_basket(customer_id)?;
        check(basket.customer_id() == customer_id, || {
            "customer_id".to_string()
        })?;
        check(basket.item_count() == 1, || "item_count".to_string())
    }
}

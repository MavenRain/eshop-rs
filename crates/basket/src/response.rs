//! Response DTOs serialized to JSON response bodies.

use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::basket::CustomerBasket;
use crate::basket_item::BasketItem;

/// `GET /api/basket/{customer_id}` response body.
#[derive(Debug, Clone, Serialize)]
pub struct BasketResponse {
    customer_id: Uuid,
    items: Vec<BasketItemResponse>,
}

impl From<&CustomerBasket> for BasketResponse {
    fn from(basket: &CustomerBasket) -> Self {
        Self {
            customer_id: basket.customer_id().into_uuid(),
            items: basket
                .items()
                .iter()
                .map(BasketItemResponse::from)
                .collect(),
        }
    }
}

/// One element of [`BasketResponse::items`].
#[derive(Debug, Clone, Serialize)]
pub struct BasketItemResponse {
    id: Uuid,
    product_id: i32,
    product_name: String,
    unit_price: Decimal,
    old_unit_price: Option<Decimal>,
    quantity: u32,
    picture_url: Option<String>,
}

impl From<&BasketItem> for BasketItemResponse {
    fn from(item: &BasketItem) -> Self {
        Self {
            id: item.id().into_uuid(),
            product_id: item.product_id().into(),
            product_name: item.product_name().as_str().to_string(),
            unit_price: item.unit_price().into_decimal(),
            old_unit_price: item.old_unit_price().map(crate::money::Price::into_decimal),
            quantity: item.quantity().get(),
            picture_url: item.picture_url().map(|p| p.as_str().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::basket::CustomerBasket;
    use crate::basket_item::{BasketItem, BasketItemId};
    use crate::customer::CustomerId;
    use crate::error::Error;
    use crate::money::{Price, Quantity};
    use crate::product::ProductId;
    use crate::strings::{PictureUrl, ProductName};

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_basket() -> Result<CustomerBasket, Error> {
        let item = BasketItem::new(
            BasketItemId::new(),
            ProductId::from(42),
            ProductName::try_from(".NET Bot Hoodie")?,
            Price::new(Decimal::from(20))?,
            Some(Price::new(Decimal::from(25))?),
            Quantity::new(2)?,
            Some(PictureUrl::try_from("hoodie.png")?),
        );
        Ok(CustomerBasket::new(CustomerId::new(), vec![item]))
    }

    #[test]
    fn basket_response_carries_every_item_field() -> Result<(), Error> {
        let basket = sample_basket()?;
        let response = BasketResponse::from(&basket);
        check(
            response.customer_id == basket.customer_id().into_uuid(),
            || "customer_id".to_string(),
        )?;
        check(response.items.len() == 1, || "len".to_string())?;
        let item = response.items.first().ok_or(Error::Validation {
            reason: "empty items".to_string(),
        })?;
        check(item.product_id == 42, || {
            format!("product_id {}", item.product_id)
        })?;
        check(item.product_name == ".NET Bot Hoodie", || {
            format!("name {}", item.product_name)
        })?;
        check(item.quantity == 2, || format!("quantity {}", item.quantity))
    }

    #[test]
    fn basket_response_serializes_to_json() -> Result<(), Error> {
        let basket = sample_basket()?;
        let response = BasketResponse::from(&basket);
        let json = serde_json::to_string(&response)?;
        check(json.contains("\"customer_id\""), || {
            format!("missing customer_id: {json}")
        })?;
        check(json.contains("\"unit_price\":\"20\""), || {
            format!("missing unit_price: {json}")
        })
    }
}

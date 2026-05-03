//! Response DTOs serialized to JSON response bodies.
//!
//! Fields are private; serde derives expand in the same module and have
//! access to them.  External callers construct via [`From`] /
//! [`CreateOrderResponse::new`] and never see the field names directly.

use chrono::{DateTime, Utc};
use ordering_domain::{BuyerId, Order, OrderId, OrderItem, PaymentMethodId};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

/// `POST /orders` response body.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderResponse {
    order_id: Uuid,
}

impl CreateOrderResponse {
    /// Wrap a fresh aggregate id for the JSON response.
    #[must_use]
    pub fn new(order_id: OrderId) -> Self {
        Self {
            order_id: order_id.into_uuid(),
        }
    }
}

/// `GET /orders/:id` response body.
#[derive(Debug, Clone, Serialize)]
pub struct GetOrderResponse {
    order_id: Uuid,
    order_date: DateTime<Utc>,
    order_status: String,
    description: String,
    buyer_id: Option<Uuid>,
    payment_method_id: Option<Uuid>,
    street: String,
    city: String,
    state: String,
    country: String,
    zip_code: String,
    items: Vec<OrderItemResponse>,
}

impl From<&Order> for GetOrderResponse {
    fn from(order: &Order) -> Self {
        let items = order
            .order_items()
            .iter()
            .map(OrderItemResponse::from)
            .collect();
        Self {
            order_id: order.id().into_uuid(),
            order_date: order.order_date().into_inner(),
            order_status: order.order_status().to_string(),
            description: order.description().as_str().to_string(),
            buyer_id: order.buyer_id().map(BuyerId::into_uuid),
            payment_method_id: order.payment_id().map(PaymentMethodId::into_uuid),
            street: order.address().street().as_str().to_string(),
            city: order.address().city().as_str().to_string(),
            state: order.address().state().as_str().to_string(),
            country: order.address().country().as_str().to_string(),
            zip_code: order.address().zip_code().as_str().to_string(),
            items,
        }
    }
}

/// One line item inside [`GetOrderResponse`].
#[derive(Debug, Clone, Serialize)]
pub struct OrderItemResponse {
    item_id: Uuid,
    product_id: i32,
    product_name: String,
    picture_url: String,
    unit_price: Decimal,
    discount: Decimal,
    units: u32,
}

impl From<&OrderItem> for OrderItemResponse {
    fn from(item: &OrderItem) -> Self {
        Self {
            item_id: item.id().into_uuid(),
            product_id: i32::from(item.product_id()),
            product_name: item.product_name().as_str().to_string(),
            picture_url: item.picture_url().as_str().to_string(),
            unit_price: item.unit_price().money().into(),
            discount: item.discount().money().into(),
            units: item.units().get(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use ordering_domain::{
        Address, CardExpiration, CardHolderName, CardNumber, CardType, City, Country, Discount,
        Money, OrderItem, OrderItemId, OrderStatus, PictureUrl, ProductId, ProductName,
        SecurityNumber, State, Street, UnitPrice, Units, UserId, UserName, ZipCode,
    };

    use crate::error::Error;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_order() -> Result<Order, Error> {
        let now = Utc::now();
        let address = Address::new(
            Street::try_from("100 Main St")?,
            City::try_from("Atlanta")?,
            State::try_from("GA")?,
            Country::try_from("USA")?,
            ZipCode::try_from("30301")?,
        );
        let order = Order::submit(
            OrderId::new(),
            UserId::try_from("user-1")?,
            UserName::try_from("Alice")?,
            address,
            CardType::Visa,
            CardNumber::try_from("4111111111111111")?,
            SecurityNumber::try_from("123")?,
            CardHolderName::try_from("Alice Buyer")?,
            CardExpiration::new(now + Duration::days(365), now)?,
            None,
            None,
        );
        let item = OrderItem::new(
            OrderItemId::new(),
            ProductId::from(7),
            ProductName::try_from("thing")?,
            PictureUrl::try_from("https://example.com/thing.png")?,
            UnitPrice::from(Money::from(rust_decimal::Decimal::new(2500, 2))),
            Discount::zero(),
            Units::new(1)?,
        )?;
        order.add_order_item(item).map_err(Error::from)
    }

    #[test]
    fn create_order_response_wraps_id() -> Result<(), Error> {
        let id = OrderId::new();
        let response = CreateOrderResponse::new(id);
        let body = serde_json::to_string(&response)?;
        let needle = id.into_uuid().to_string();
        check(body.contains(&needle), || {
            format!("response body {body} did not contain id {needle}")
        })
    }

    #[test]
    fn get_order_response_projects_all_fields() -> Result<(), Error> {
        let order = sample_order()?;
        let response = GetOrderResponse::from(&order);
        check(response.order_id == order.id().into_uuid(), || {
            "order_id mismatch".to_string()
        })?;
        check(
            response.order_status == OrderStatus::Submitted.to_string(),
            || format!("status mismatch: {}", response.order_status),
        )?;
        check(response.items.len() == 1, || {
            format!("expected 1 item, got {}", response.items.len())
        })?;
        check(response.zip_code == "30301", || {
            format!("zip_code mismatch: {}", response.zip_code)
        })
    }

    #[test]
    fn order_item_response_projects_decimals_and_units() -> Result<(), Error> {
        let order = sample_order()?;
        let item = order
            .order_items()
            .first()
            .ok_or_else(|| Error::Validation {
                reason: "fixture missing item".to_string(),
            })?;
        let response = OrderItemResponse::from(item);
        check(response.units == 1, || format!("units {}", response.units))?;
        check(response.product_id == 7, || {
            format!("product_id {}", response.product_id)
        })
    }
}

//! Request DTOs deserialized from JSON request bodies.
//!
//! Fields are private; serde derives expand in the same module and have
//! access to them.  Public projection methods convert each request into
//! the corresponding domain type, validating along the way.

use chrono::{DateTime, Utc};
use ordering_domain::{
    Address, CardExpiration, CardHolderName, CardNumber, CardType, City, Country, Discount, Money,
    Order, OrderId, OrderItem, OrderItemId, PictureUrl, ProductId, ProductName, SecurityNumber,
    State as DomainState, Street, UnitPrice, Units, UserId, UserName, ZipCode,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;

/// `POST /orders` body.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    user_id: String,
    user_name: String,
    street: String,
    city: String,
    state: String,
    country: String,
    zip_code: String,
    card_number: String,
    card_security_number: String,
    card_holder_name: String,
    card_expiration: DateTime<Utc>,
    card_type: String,
    items: Vec<CreateOrderItemRequest>,
}

impl CreateOrderRequest {
    /// Project this request into a fresh [`Order`] aggregate.
    ///
    /// `now` is the wall-clock used to validate `card_expiration`; the
    /// caller is expected to pass `chrono::Utc::now()` in production
    /// and a fixed value in tests.
    ///
    /// # Errors
    /// Returns [`Error::Validation`] for malformed strings, [`Error::Domain`]
    /// for domain invariant failures (expired card, duplicate item, etc.).
    pub fn try_into_order(self, now: DateTime<Utc>) -> Result<Order, Error> {
        let address = Address::new(
            Street::try_from(self.street)?,
            City::try_from(self.city)?,
            DomainState::try_from(self.state)?,
            Country::try_from(self.country)?,
            ZipCode::try_from(self.zip_code)?,
        );
        let card_type = parse_card_type(&self.card_type)?;
        let card_expiration = CardExpiration::new(self.card_expiration, now)?;
        let seed = Order::submit(
            OrderId::new(),
            UserId::try_from(self.user_id)?,
            UserName::try_from(self.user_name)?,
            address,
            card_type,
            CardNumber::try_from(self.card_number)?,
            SecurityNumber::try_from(self.card_security_number)?,
            CardHolderName::try_from(self.card_holder_name)?,
            card_expiration,
            None,
            None,
        );
        self.items.into_iter().try_fold(seed, |acc, item| {
            let order_item = item.try_into_order_item()?;
            acc.add_order_item(order_item).map_err(Error::from)
        })
    }
}

/// One line item inside [`CreateOrderRequest`].
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderItemRequest {
    product_id: i32,
    product_name: String,
    picture_url: String,
    unit_price: Decimal,
    discount: Decimal,
    units: u32,
}

impl CreateOrderItemRequest {
    fn try_into_order_item(self) -> Result<OrderItem, Error> {
        OrderItem::new(
            OrderItemId::new(),
            ProductId::from(self.product_id),
            ProductName::try_from(self.product_name)?,
            PictureUrl::try_from(self.picture_url)?,
            UnitPrice::from(Money::from(self.unit_price)),
            Discount::new(Money::from(self.discount))?,
            Units::new(self.units)?,
        )
        .map_err(Error::from)
    }
}

/// `PUT /orders/cancel` body.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderRequest {
    order_id: Uuid,
}

impl CancelOrderRequest {
    /// Domain id parsed from the JSON body.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        OrderId::from(self.order_id)
    }
}

/// `PUT /orders/ship` body.
#[derive(Debug, Clone, Deserialize)]
pub struct ShipOrderRequest {
    order_id: Uuid,
}

impl ShipOrderRequest {
    /// Domain id parsed from the JSON body.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        OrderId::from(self.order_id)
    }
}

fn parse_card_type(s: &str) -> Result<CardType, Error> {
    match s {
        "Amex" => Ok(CardType::Amex),
        "Visa" => Ok(CardType::Visa),
        "MasterCard" => Ok(CardType::MasterCard),
        other => Err(Error::Validation {
            reason: format!("unknown card type: {other}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use ordering_domain::OrderStatus;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    fn sample_now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .map_or_else(|_| Utc::now(), |t| t.with_timezone(&Utc))
    }

    fn valid_request() -> CreateOrderRequest {
        let now = sample_now();
        CreateOrderRequest {
            user_id: "user-1".to_string(),
            user_name: "Alice".to_string(),
            street: "100 Main St".to_string(),
            city: "Atlanta".to_string(),
            state: "GA".to_string(),
            country: "USA".to_string(),
            zip_code: "30301".to_string(),
            card_number: "4111111111111111".to_string(),
            card_security_number: "123".to_string(),
            card_holder_name: "Alice Buyer".to_string(),
            card_expiration: now + Duration::days(365),
            card_type: "Visa".to_string(),
            items: vec![CreateOrderItemRequest {
                product_id: 7,
                product_name: "thing".to_string(),
                picture_url: "https://example.com/thing.png".to_string(),
                unit_price: Decimal::new(2500, 2),
                discount: Decimal::ZERO,
                units: 1,
            }],
        }
    }

    #[test]
    fn parse_card_type_known_variants() -> Result<(), Error> {
        check(matches!(parse_card_type("Amex")?, CardType::Amex), || {
            "Amex".to_string()
        })?;
        check(matches!(parse_card_type("Visa")?, CardType::Visa), || {
            "Visa".to_string()
        })?;
        check(
            matches!(parse_card_type("MasterCard")?, CardType::MasterCard),
            || "MasterCard".to_string(),
        )
    }

    #[test]
    fn parse_card_type_unknown_rejected() -> Result<(), Error> {
        let outcome = parse_card_type("Discover");
        check(matches!(outcome, Err(Error::Validation { .. })), || {
            format!("expected Validation err, got {outcome:?}")
        })
    }

    #[test]
    fn try_into_order_happy_path() -> Result<(), Error> {
        let order = valid_request().try_into_order(sample_now())?;
        check(order.order_status() == OrderStatus::Submitted, || {
            format!("expected Submitted, got {}", order.order_status())
        })?;
        check(order.order_items().len() == 1, || {
            format!("expected 1 item, got {}", order.order_items().len())
        })
    }

    #[test]
    fn try_into_order_unknown_card_type_rejected() -> Result<(), Error> {
        let request = CreateOrderRequest {
            card_type: "Discover".to_string(),
            ..valid_request()
        };
        let outcome = request.try_into_order(sample_now());
        check(matches!(outcome, Err(Error::Validation { .. })), || {
            format!("expected Validation err, got {outcome:?}")
        })
    }

    #[test]
    fn try_into_order_expired_card_rejected() -> Result<(), Error> {
        let now = sample_now();
        let request = CreateOrderRequest {
            card_expiration: now - Duration::days(1),
            ..valid_request()
        };
        let outcome = request.try_into_order(now);
        check(matches!(outcome, Err(Error::Domain { .. })), || {
            format!("expected Domain err, got {outcome:?}")
        })
    }

    #[test]
    fn cancel_order_request_projects_id() -> Result<(), Error> {
        let raw = Uuid::new_v4();
        let request = CancelOrderRequest { order_id: raw };
        check(request.order_id().into_uuid() == raw, || {
            "round-trip mismatch".to_string()
        })
    }

    #[test]
    fn ship_order_request_projects_id() -> Result<(), Error> {
        let raw = Uuid::new_v4();
        let request = ShipOrderRequest { order_id: raw };
        check(request.order_id().into_uuid() == raw, || {
            "round-trip mismatch".to_string()
        })
    }

    #[test]
    fn deserialize_create_order_request() -> Result<(), Error> {
        let body = r#"{
            "user_id": "u",
            "user_name": "n",
            "street": "s",
            "city": "c",
            "state": "GA",
            "country": "US",
            "zip_code": "z",
            "card_number": "4111111111111111",
            "card_security_number": "1",
            "card_holder_name": "h",
            "card_expiration": "2030-01-01T00:00:00Z",
            "card_type": "Visa",
            "items": []
        }"#;
        let parsed: CreateOrderRequest = serde_json::from_str(body)?;
        check(parsed.items.is_empty(), || "items not empty".to_string())
    }
}

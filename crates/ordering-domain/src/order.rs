//! [`Order`] aggregate root.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::address::Address;
use crate::buyer::BuyerId;
use crate::card_type::CardType;
use crate::error::Error;
use crate::event::{
    DomainEvent, OrderCancelledEvent, OrderShippedEvent, OrderStartedEvent,
    OrderStatusChangedToAwaitingValidationEvent, OrderStatusChangedToPaidEvent,
    OrderStatusChangedToStockConfirmedEvent,
};
use crate::money::Money;
use crate::order_item::{OrderItem, ProductId};
use crate::order_status::OrderStatus;
use crate::payment_method::{CardExpiration, PaymentMethodId};
use crate::strings::{CardHolderName, CardNumber, SecurityNumber, UserId, UserName};

/// Identifier for an [`Order`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct OrderId(Uuid);

impl OrderId {
    /// Generate a fresh identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Underlying [`Uuid`].
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for OrderId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<OrderId> for Uuid {
    fn from(id: OrderId) -> Self {
        id.0
    }
}

/// Timestamp at which an order was placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct OrderDate(DateTime<Utc>);

impl OrderDate {
    /// Capture the current UTC time.
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Underlying timestamp.
    #[must_use]
    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }
}

impl From<DateTime<Utc>> for OrderDate {
    fn from(t: DateTime<Utc>) -> Self {
        Self(t)
    }
}

impl From<OrderDate> for DateTime<Utc> {
    fn from(d: OrderDate) -> Self {
        d.0
    }
}

/// Free-form order description.  May be empty (default for newly submitted
/// orders; populated on status transitions).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Description(String);

impl Description {
    /// An empty description.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Description {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Description {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Description> for String {
    fn from(d: Description) -> Self {
        d.0
    }
}

/// Aggregate root tracking an order through its lifecycle.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    id: OrderId,
    user_id: UserId,
    order_date: OrderDate,
    address: Address,
    buyer_id: Option<BuyerId>,
    order_status: OrderStatus,
    description: Description,
    order_items: Vec<OrderItem>,
    payment_id: Option<PaymentMethodId>,
    domain_events: Vec<DomainEvent>,
}

impl Order {
    /// Rehydrate an order from persisted state.  No invariant checks are
    /// performed beyond what the caller's newtypes already enforce; the
    /// `domain_events` list is reset to empty.  Use this only at the
    /// persistence boundary.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn rehydrate(
        id: OrderId,
        user_id: UserId,
        order_date: OrderDate,
        address: Address,
        buyer_id: Option<BuyerId>,
        order_status: OrderStatus,
        description: Description,
        order_items: Vec<OrderItem>,
        payment_id: Option<PaymentMethodId>,
    ) -> Self {
        Self {
            id,
            user_id,
            order_date,
            address,
            buyer_id,
            order_status,
            description,
            order_items,
            payment_id,
            domain_events: Vec::new(),
        }
    }

    /// Submit a new order, emitting [`DomainEvent::OrderStarted`].  The
    /// resulting order has [`OrderStatus::Submitted`] and an empty item list.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn submit(
        id: OrderId,
        user_id: UserId,
        user_name: UserName,
        address: Address,
        card_type: CardType,
        card_number: CardNumber,
        card_security_number: SecurityNumber,
        card_holder_name: CardHolderName,
        card_expiration: CardExpiration,
        buyer_id: Option<BuyerId>,
        payment_method_id: Option<PaymentMethodId>,
    ) -> Self {
        let event = DomainEvent::OrderStarted(OrderStartedEvent::new(
            id,
            user_id.clone(),
            user_name,
            card_type,
            card_number,
            card_security_number,
            card_holder_name,
            card_expiration,
        ));
        Self {
            id,
            user_id,
            order_date: OrderDate::now(),
            address,
            buyer_id,
            order_status: OrderStatus::Submitted,
            description: Description::empty(),
            order_items: Vec::new(),
            payment_id: payment_method_id,
            domain_events: vec![event],
        }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> OrderId {
        self.id
    }

    /// External identity provider's user id (the order's owner).
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Order date.
    #[must_use]
    pub fn order_date(&self) -> OrderDate {
        self.order_date
    }

    /// Address.
    #[must_use]
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Buyer identifier.
    #[must_use]
    pub fn buyer_id(&self) -> Option<BuyerId> {
        self.buyer_id
    }

    /// Order status.
    #[must_use]
    pub fn order_status(&self) -> OrderStatus {
        self.order_status
    }

    /// Description.
    #[must_use]
    pub fn description(&self) -> &Description {
        &self.description
    }

    /// Read-only view of items.
    #[must_use]
    pub fn order_items(&self) -> &[OrderItem] {
        &self.order_items
    }

    /// Payment method identifier.
    #[must_use]
    pub fn payment_id(&self) -> Option<PaymentMethodId> {
        self.payment_id
    }

    /// Read-only view of pending domain events.
    #[must_use]
    pub fn domain_events(&self) -> &[DomainEvent] {
        &self.domain_events
    }

    /// Drain the pending domain events, returning them and a fresh [`Order`]
    /// with no events.
    #[must_use]
    pub fn take_events(self) -> (Self, Vec<DomainEvent>) {
        let events = self.domain_events;
        (
            Self {
                domain_events: Vec::new(),
                ..self
            },
            events,
        )
    }

    /// Add or merge a line item.  Items with a matching [`ProductId`] are
    /// merged: units summed, discount kept at the maximum of the two.
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidUnits)` if merging would overflow the unit
    /// count.  Returns `Err(Error::DiscountExceedsTotal)` if a merged
    /// discount would exceed the line total.  Returns
    /// `Err(Error::DecimalOverflow)` on arithmetic overflow.
    pub fn add_order_item(self, item: OrderItem) -> Result<Self, Error> {
        let target = item.product_id();
        let existing = self.order_items.iter().any(|o| o.product_id() == target);
        let new_items: Result<Vec<OrderItem>, Error> = if existing {
            self.order_items
                .iter()
                .map(|o| {
                    if o.product_id() == target {
                        o.clone()
                            .add_units(item.units())
                            .and_then(|merged| merged.set_max_discount(item.discount()))
                    } else {
                        Ok(o.clone())
                    }
                })
                .collect()
        } else {
            Ok(self
                .order_items
                .iter()
                .cloned()
                .chain(core::iter::once(item))
                .collect())
        };
        new_items.map(|items| Self {
            order_items: items,
            ..self
        })
    }

    /// Record that a payment method has been verified for this order.
    #[must_use]
    pub fn set_payment_method_verified(
        self,
        buyer_id: BuyerId,
        payment_id: PaymentMethodId,
    ) -> Self {
        Self {
            buyer_id: Some(buyer_id),
            payment_id: Some(payment_id),
            ..self
        }
    }

    /// Transition to [`OrderStatus::AwaitingValidation`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status is
    /// not [`OrderStatus::Submitted`].
    pub fn set_awaiting_validation_status(self) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::Submitted => {
                let event = DomainEvent::OrderStatusChangedToAwaitingValidation(
                    OrderStatusChangedToAwaitingValidationEvent::new(
                        self.id,
                        self.order_items.clone(),
                    ),
                );
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::AwaitingValidation,
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::AwaitingValidation
            | OrderStatus::StockConfirmed
            | OrderStatus::Paid
            | OrderStatus::Shipped
            | OrderStatus::Cancelled) => Err(Error::InvalidStatusTransition {
                from,
                to: OrderStatus::AwaitingValidation,
            }),
        }
    }

    /// Transition to [`OrderStatus::StockConfirmed`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status is
    /// not [`OrderStatus::AwaitingValidation`].
    pub fn set_stock_confirmed_status(self) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::AwaitingValidation => {
                let event = DomainEvent::OrderStatusChangedToStockConfirmed(
                    OrderStatusChangedToStockConfirmedEvent::new(self.id),
                );
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::StockConfirmed,
                    description: Description::from(
                        "All the items were confirmed with available stock.",
                    ),
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::Submitted
            | OrderStatus::StockConfirmed
            | OrderStatus::Paid
            | OrderStatus::Shipped
            | OrderStatus::Cancelled) => Err(Error::InvalidStatusTransition {
                from,
                to: OrderStatus::StockConfirmed,
            }),
        }
    }

    /// Transition to [`OrderStatus::Paid`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status is
    /// not [`OrderStatus::StockConfirmed`].
    pub fn set_paid_status(self) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::StockConfirmed => {
                let event = DomainEvent::OrderStatusChangedToPaid(
                    OrderStatusChangedToPaidEvent::new(self.id, self.order_items.clone()),
                );
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::Paid,
                    description: Description::from(
                        "The payment was performed at a simulated bank.",
                    ),
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::Submitted
            | OrderStatus::AwaitingValidation
            | OrderStatus::Paid
            | OrderStatus::Shipped
            | OrderStatus::Cancelled) => Err(Error::InvalidStatusTransition {
                from,
                to: OrderStatus::Paid,
            }),
        }
    }

    /// Transition to [`OrderStatus::Shipped`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status is
    /// not [`OrderStatus::Paid`].
    pub fn set_shipped_status(self) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::Paid => {
                let event = DomainEvent::OrderShipped(OrderShippedEvent::new(self.id));
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::Shipped,
                    description: Description::from("The order was shipped."),
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::Submitted
            | OrderStatus::AwaitingValidation
            | OrderStatus::StockConfirmed
            | OrderStatus::Shipped
            | OrderStatus::Cancelled) => Err(Error::InvalidStatusTransition {
                from,
                to: OrderStatus::Shipped,
            }),
        }
    }

    /// Transition to [`OrderStatus::Cancelled`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status
    /// is [`OrderStatus::Paid`], [`OrderStatus::Shipped`], or already
    /// [`OrderStatus::Cancelled`].
    pub fn set_cancelled_status(self) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::Submitted
            | OrderStatus::AwaitingValidation
            | OrderStatus::StockConfirmed => {
                let event = DomainEvent::OrderCancelled(OrderCancelledEvent::new(self.id));
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::Cancelled,
                    description: Description::from("The order was cancelled."),
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::Paid | OrderStatus::Shipped | OrderStatus::Cancelled) => {
                Err(Error::InvalidStatusTransition {
                    from,
                    to: OrderStatus::Cancelled,
                })
            }
        }
    }

    /// Cancel an order whose stock check rejected one or more product ids.
    /// The description is set to a human-readable list of the rejected
    /// product names.
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidStatusTransition)` if the current status
    /// is not [`OrderStatus::AwaitingValidation`].
    pub fn set_cancelled_when_stock_is_rejected(
        self,
        rejected_product_ids: &[ProductId],
    ) -> Result<Self, Error> {
        match self.order_status {
            OrderStatus::AwaitingValidation => {
                let names: Vec<&str> = self
                    .order_items
                    .iter()
                    .filter(|o| rejected_product_ids.contains(&o.product_id()))
                    .map(|o| o.product_name().as_str())
                    .collect();
                let joined = names.join(", ");
                let description =
                    Description::from(format!("The product items don't have stock: ({joined})."));
                let event = DomainEvent::OrderCancelled(OrderCancelledEvent::new(self.id));
                let domain_events = self
                    .domain_events
                    .iter()
                    .cloned()
                    .chain(core::iter::once(event))
                    .collect();
                Ok(Self {
                    order_status: OrderStatus::Cancelled,
                    description,
                    domain_events,
                    ..self
                })
            }
            from @ (OrderStatus::Submitted
            | OrderStatus::StockConfirmed
            | OrderStatus::Paid
            | OrderStatus::Shipped
            | OrderStatus::Cancelled) => Err(Error::InvalidStatusTransition {
                from,
                to: OrderStatus::Cancelled,
            }),
        }
    }

    /// Total of all line items.
    ///
    /// # Errors
    /// Returns `Err(Error::DecimalOverflow)` on arithmetic overflow.
    pub fn total(&self) -> Result<Money, Error> {
        self.order_items
            .iter()
            .try_fold(Money::zero(), |acc, item| {
                item.line_total().and_then(|t| acc.checked_add(t))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card_type::CardType;
    use crate::money::{Discount, UnitPrice};
    use crate::order_item::{OrderItemId, Units};
    use crate::strings::{City, Country, PictureUrl, ProductName, State, Street, ZipCode};
    use chrono::Duration;
    use rust_decimal::Decimal;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    fn sample_address() -> Result<Address, Error> {
        Ok(Address::new(
            Street::try_from("100 Main St")?,
            City::try_from("Atlanta")?,
            State::try_from("GA")?,
            Country::try_from("USA")?,
            ZipCode::try_from("30301")?,
        ))
    }

    fn sample_card_expiration() -> Result<CardExpiration, Error> {
        let now = Utc::now();
        CardExpiration::new(now + Duration::days(365), now)
    }

    fn submitted_order() -> Result<Order, Error> {
        Ok(Order::submit(
            OrderId::new(),
            UserId::try_from("user-1")?,
            UserName::try_from("Alice")?,
            sample_address()?,
            CardType::Visa,
            CardNumber::try_from("4111111111111111")?,
            SecurityNumber::try_from("123")?,
            CardHolderName::try_from("Alice Buyer")?,
            sample_card_expiration()?,
            None,
            None,
        ))
    }

    fn sample_item(product_id: i32, units: u32, unit_price: i64) -> Result<OrderItem, Error> {
        OrderItem::new(
            OrderItemId::new(),
            ProductId::from(product_id),
            ProductName::try_from(format!("product-{product_id}"))?,
            PictureUrl::try_from(format!("https://example.com/{product_id}.png"))?,
            UnitPrice::from(Money::from(Decimal::from(unit_price))),
            Discount::zero(),
            Units::new(units)?,
        )
    }

    #[test]
    fn submit_yields_submitted_with_event() -> Result<(), Error> {
        let order = submitted_order()?;
        check(order.order_status() == OrderStatus::Submitted, || {
            format!("expected Submitted, got {}", order.order_status())
        })?;
        check(order.domain_events().len() == 1, || {
            format!("expected 1 event, got {}", order.domain_events().len())
        })?;
        check(
            matches!(
                order.domain_events().first(),
                Some(DomainEvent::OrderStarted(_))
            ),
            || "expected OrderStarted event".to_string(),
        )
    }

    #[test]
    fn happy_path_submitted_to_shipped() -> Result<(), Error> {
        let order = submitted_order()?
            .set_awaiting_validation_status()?
            .set_stock_confirmed_status()?
            .set_paid_status()?
            .set_shipped_status()?;
        check(order.order_status() == OrderStatus::Shipped, || {
            format!("expected Shipped, got {}", order.order_status())
        })?;
        check(order.domain_events().len() == 5, || {
            format!("expected 5 events, got {}", order.domain_events().len())
        })
    }

    #[test]
    fn invalid_transition_returns_err() -> Result<(), Error> {
        let order = submitted_order()?;
        let outcome = order.set_paid_status();
        check(
            matches!(
                outcome,
                Err(Error::InvalidStatusTransition {
                    from: OrderStatus::Submitted,
                    to: OrderStatus::Paid,
                })
            ),
            || format!("expected InvalidStatusTransition, got {outcome:?}"),
        )
    }

    #[test]
    fn cancel_after_paid_rejected() -> Result<(), Error> {
        let order = submitted_order()?
            .set_awaiting_validation_status()?
            .set_stock_confirmed_status()?
            .set_paid_status()?;
        let outcome = order.set_cancelled_status();
        check(
            matches!(
                outcome,
                Err(Error::InvalidStatusTransition {
                    from: OrderStatus::Paid,
                    to: OrderStatus::Cancelled,
                })
            ),
            || format!("expected InvalidStatusTransition Paid->Cancelled, got {outcome:?}"),
        )
    }

    #[test]
    fn add_distinct_items_appends() -> Result<(), Error> {
        let order = submitted_order()?
            .add_order_item(sample_item(1, 2, 10)?)?
            .add_order_item(sample_item(2, 3, 5)?)?;
        check(order.order_items().len() == 2, || {
            format!("expected 2 items, got {}", order.order_items().len())
        })
    }

    #[test]
    fn add_same_product_merges_units() -> Result<(), Error> {
        let order = submitted_order()?
            .add_order_item(sample_item(1, 2, 10)?)?
            .add_order_item(sample_item(1, 3, 10)?)?;
        check(order.order_items().len() == 1, || {
            format!("expected 1 merged item, got {}", order.order_items().len())
        })?;
        let units =
            order
                .order_items()
                .first()
                .map(OrderItem::units)
                .ok_or(Error::InvariantViolated {
                    reason: "missing item".to_string(),
                })?;
        check(units.get() == 5, || {
            format!("expected 5 units, got {}", units.get())
        })
    }

    #[test]
    fn total_sums_line_totals() -> Result<(), Error> {
        let order = submitted_order()?
            .add_order_item(sample_item(1, 2, 10)?)?
            .add_order_item(sample_item(2, 3, 5)?)?;
        let total = order.total()?;
        check(total == Money::from(Decimal::from(35)), || {
            format!("expected 35, got {total}")
        })
    }

    #[test]
    fn stock_rejection_cancels_with_description() -> Result<(), Error> {
        let order = submitted_order()?
            .add_order_item(sample_item(1, 2, 10)?)?
            .add_order_item(sample_item(2, 3, 5)?)?
            .set_awaiting_validation_status()?
            .set_cancelled_when_stock_is_rejected(&[ProductId::from(1)])?;
        check(order.order_status() == OrderStatus::Cancelled, || {
            format!("expected Cancelled, got {}", order.order_status())
        })?;
        let desc = order.description().as_str();
        check(desc.contains("product-1"), || {
            format!("expected rejected name in description, got {desc}")
        })
    }
}

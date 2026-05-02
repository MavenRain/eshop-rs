//! Domain events emitted by the Ordering bounded context.

use crate::buyer::BuyerId;
use crate::card_type::CardType;
use crate::order::OrderId;
use crate::order_item::OrderItem;
use crate::payment_method::{CardExpiration, PaymentMethodId};
use crate::strings::{CardHolderName, CardNumber, SecurityNumber, UserId, UserName};

/// Domain events emitted by Ordering aggregates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Order has been created and submitted.
    OrderStarted(OrderStartedEvent),
    /// Status moved to [`OrderStatus::AwaitingValidation`](crate::order_status::OrderStatus::AwaitingValidation).
    OrderStatusChangedToAwaitingValidation(OrderStatusChangedToAwaitingValidationEvent),
    /// Status moved to [`OrderStatus::StockConfirmed`](crate::order_status::OrderStatus::StockConfirmed).
    OrderStatusChangedToStockConfirmed(OrderStatusChangedToStockConfirmedEvent),
    /// Status moved to [`OrderStatus::Paid`](crate::order_status::OrderStatus::Paid).
    OrderStatusChangedToPaid(OrderStatusChangedToPaidEvent),
    /// Status moved to [`OrderStatus::Shipped`](crate::order_status::OrderStatus::Shipped).
    OrderShipped(OrderShippedEvent),
    /// Status moved to [`OrderStatus::Cancelled`](crate::order_status::OrderStatus::Cancelled).
    OrderCancelled(OrderCancelledEvent),
    /// A buyer's payment method has been verified or registered for an order.
    BuyerPaymentMethodVerified(BuyerPaymentMethodVerifiedEvent),
}

/// Payload of [`DomainEvent::OrderStarted`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderStartedEvent {
    order_id: OrderId,
    user_id: UserId,
    user_name: UserName,
    card_type: CardType,
    card_number: CardNumber,
    card_security_number: SecurityNumber,
    card_holder_name: CardHolderName,
    card_expiration: CardExpiration,
}

impl OrderStartedEvent {
    /// Construct.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        order_id: OrderId,
        user_id: UserId,
        user_name: UserName,
        card_type: CardType,
        card_number: CardNumber,
        card_security_number: SecurityNumber,
        card_holder_name: CardHolderName,
        card_expiration: CardExpiration,
    ) -> Self {
        Self {
            order_id,
            user_id,
            user_name,
            card_type,
            card_number,
            card_security_number,
            card_holder_name,
            card_expiration,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// User identifier.
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// User display name.
    #[must_use]
    pub fn user_name(&self) -> &UserName {
        &self.user_name
    }

    /// Card brand.
    #[must_use]
    pub fn card_type(&self) -> CardType {
        self.card_type
    }

    /// Card number.
    #[must_use]
    pub fn card_number(&self) -> &CardNumber {
        &self.card_number
    }

    /// Card security number.
    #[must_use]
    pub fn card_security_number(&self) -> &SecurityNumber {
        &self.card_security_number
    }

    /// Cardholder name.
    #[must_use]
    pub fn card_holder_name(&self) -> &CardHolderName {
        &self.card_holder_name
    }

    /// Card expiration.
    #[must_use]
    pub fn card_expiration(&self) -> CardExpiration {
        self.card_expiration
    }
}

/// Payload of [`DomainEvent::OrderStatusChangedToAwaitingValidation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderStatusChangedToAwaitingValidationEvent {
    order_id: OrderId,
    order_items: Vec<OrderItem>,
}

impl OrderStatusChangedToAwaitingValidationEvent {
    /// Construct.
    #[must_use]
    pub fn new(order_id: OrderId, order_items: Vec<OrderItem>) -> Self {
        Self {
            order_id,
            order_items,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// Snapshot of the order items at transition time.
    #[must_use]
    pub fn order_items(&self) -> &[OrderItem] {
        &self.order_items
    }
}

/// Payload of [`DomainEvent::OrderStatusChangedToStockConfirmed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderStatusChangedToStockConfirmedEvent {
    order_id: OrderId,
}

impl OrderStatusChangedToStockConfirmedEvent {
    /// Construct.
    #[must_use]
    pub fn new(order_id: OrderId) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }
}

/// Payload of [`DomainEvent::OrderStatusChangedToPaid`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderStatusChangedToPaidEvent {
    order_id: OrderId,
    order_items: Vec<OrderItem>,
}

impl OrderStatusChangedToPaidEvent {
    /// Construct.
    #[must_use]
    pub fn new(order_id: OrderId, order_items: Vec<OrderItem>) -> Self {
        Self {
            order_id,
            order_items,
        }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// Snapshot of the order items at transition time.
    #[must_use]
    pub fn order_items(&self) -> &[OrderItem] {
        &self.order_items
    }
}

/// Payload of [`DomainEvent::OrderShipped`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderShippedEvent {
    order_id: OrderId,
}

impl OrderShippedEvent {
    /// Construct.
    #[must_use]
    pub fn new(order_id: OrderId) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }
}

/// Payload of [`DomainEvent::OrderCancelled`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderCancelledEvent {
    order_id: OrderId,
}

impl OrderCancelledEvent {
    /// Construct.
    #[must_use]
    pub fn new(order_id: OrderId) -> Self {
        Self { order_id }
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }
}

/// Payload of [`DomainEvent::BuyerPaymentMethodVerified`].
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyerPaymentMethodVerifiedEvent {
    buyer_id: BuyerId,
    payment_method_id: PaymentMethodId,
    order_id: OrderId,
}

impl BuyerPaymentMethodVerifiedEvent {
    /// Construct.
    #[must_use]
    pub fn new(buyer_id: BuyerId, payment_method_id: PaymentMethodId, order_id: OrderId) -> Self {
        Self {
            buyer_id,
            payment_method_id,
            order_id,
        }
    }

    /// Buyer identifier.
    #[must_use]
    pub fn buyer_id(&self) -> BuyerId {
        self.buyer_id
    }

    /// Payment method identifier.
    #[must_use]
    pub fn payment_method_id(&self) -> PaymentMethodId {
        self.payment_method_id
    }

    /// Order identifier.
    #[must_use]
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }
}

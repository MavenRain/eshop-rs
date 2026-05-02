//! Ordering bounded context: domain types for the `eshop-rs` port.
//!
//! Type-driven core of the Ordering service.  Houses the [`Order`] and
//! [`Buyer`] aggregates, value objects ([`Address`], [`Money`],
//! [`OrderStatus`], [`CardType`]), domain events ([`DomainEvent`]), and a
//! hand-rolled [`Error`] enum.  Persistence and transport live in sibling
//! crates; this crate has no I/O.

pub mod address;
pub mod buyer;
pub mod card_type;
pub mod error;
pub mod event;
pub mod money;
pub mod order;
pub mod order_item;
pub mod order_status;
pub mod payment_method;
pub mod strings;

pub use address::Address;
pub use buyer::{Buyer, BuyerId};
pub use card_type::CardType;
pub use error::Error;
pub use event::{
    BuyerPaymentMethodVerifiedEvent, DomainEvent, OrderCancelledEvent, OrderShippedEvent,
    OrderStartedEvent, OrderStatusChangedToAwaitingValidationEvent, OrderStatusChangedToPaidEvent,
    OrderStatusChangedToStockConfirmedEvent,
};
pub use money::{Discount, Money, UnitPrice};
pub use order::{Description, Order, OrderDate, OrderId};
pub use order_item::{OrderItem, OrderItemId, ProductId, Units};
pub use order_status::OrderStatus;
pub use payment_method::{CardExpiration, PaymentMethod, PaymentMethodId};
pub use strings::{
    CardAlias, CardHolderName, CardNumber, City, Country, IdentityGuid, PictureUrl, ProductName,
    SecurityNumber, State, Street, UserId, UserName, ZipCode,
};

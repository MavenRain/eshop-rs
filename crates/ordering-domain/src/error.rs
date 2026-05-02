//! Domain error type for the Ordering bounded context.

use crate::order_status::OrderStatus;

/// Errors produced by Ordering domain operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Construction of a non-empty string failed because the input was empty.
    EmptyString { field: &'static str },
    /// `Units` was zero.
    InvalidUnits,
    /// Discount exceeded the line item total.
    DiscountExceedsTotal,
    /// Discount was negative.
    NegativeDiscount,
    /// Card expiration was not strictly in the future at construction time.
    CardExpired,
    /// Decimal arithmetic overflowed.
    DecimalOverflow,
    /// `OrderStatus` transition was rejected by the state machine.
    InvalidStatusTransition { from: OrderStatus, to: OrderStatus },
    /// An unknown numeric ID was supplied for a [`CardType`](crate::card_type::CardType).
    UnknownCardType { id: i32 },
    /// A domain invariant was violated; carries a human-readable reason.
    InvariantViolated { reason: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString { field } => write!(f, "{field} must not be empty"),
            Self::InvalidUnits => write!(f, "units must be positive"),
            Self::DiscountExceedsTotal => write!(f, "discount exceeds line total"),
            Self::NegativeDiscount => write!(f, "discount must not be negative"),
            Self::CardExpired => write!(f, "card expiration must be in the future"),
            Self::DecimalOverflow => write!(f, "decimal arithmetic overflowed"),
            Self::InvalidStatusTransition { from, to } => {
                write!(f, "cannot transition order status from {from} to {to}")
            }
            Self::UnknownCardType { id } => write!(f, "unknown card type id: {id}"),
            Self::InvariantViolated { reason } => write!(f, "invariant violated: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

//! Order lifecycle status.

use core::fmt;

/// Lifecycle states an [`Order`](crate::order::Order) can occupy.
///
/// Numeric tags returned by [`OrderStatus::tag`] mirror the upstream
/// `dotnet/eShop` enum values and are part of the persisted contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    /// Submitted; awaiting validation.
    Submitted,
    /// Awaiting stock validation.
    AwaitingValidation,
    /// Stock has been confirmed.
    StockConfirmed,
    /// Order has been paid.
    Paid,
    /// Order has been shipped.
    Shipped,
    /// Order was cancelled.
    Cancelled,
}

impl OrderStatus {
    /// Numeric tag matching upstream `dotnet/eShop`.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            Self::Submitted => 1,
            Self::AwaitingValidation => 2,
            Self::StockConfirmed => 3,
            Self::Paid => 4,
            Self::Shipped => 5,
            Self::Cancelled => 6,
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Submitted => f.write_str("Submitted"),
            Self::AwaitingValidation => f.write_str("AwaitingValidation"),
            Self::StockConfirmed => f.write_str("StockConfirmed"),
            Self::Paid => f.write_str("Paid"),
            Self::Shipped => f.write_str("Shipped"),
            Self::Cancelled => f.write_str("Cancelled"),
        }
    }
}

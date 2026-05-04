//! Price and quantity newtypes for basket items.

use core::num::NonZeroU32;

use rust_decimal::Decimal;

use crate::error::Error;

/// Non-negative price.  Mirror of [`catalog::Price`](https://docs.rs/catalog).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Price(Decimal);

impl Price {
    /// Construct from a [`Decimal`].
    ///
    /// # Errors
    /// Returns [`Error::NegativePrice`] if `value` is below zero.
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value < Decimal::ZERO {
            Err(Error::NegativePrice)
        } else {
            Ok(Self(value))
        }
    }

    /// Borrow the inner [`Decimal`] (used by mappers and DTOs).
    #[must_use]
    pub fn into_decimal(self) -> Decimal {
        self.0
    }
}

/// Strictly positive quantity of a basket item.  Zero is a domain
/// error: a basket item with zero units is the same thing as no item
/// at all and the request layer should reject it earlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Quantity(NonZeroU32);

impl Quantity {
    /// Construct from a `u32`.
    ///
    /// # Errors
    /// Returns [`Error::ZeroQuantity`] if `value` is zero.
    pub fn new(value: u32) -> Result<Self, Error> {
        NonZeroU32::new(value).map(Self).ok_or(Error::ZeroQuantity)
    }

    /// Number of units.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl From<NonZeroU32> for Quantity {
    fn from(value: NonZeroU32) -> Self {
        Self(value)
    }
}

impl From<Quantity> for u32 {
    fn from(q: Quantity) -> Self {
        q.0.get()
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

    #[test]
    fn negative_price_rejected() -> Result<(), Error> {
        let outcome = Price::new(Decimal::NEGATIVE_ONE);
        check(matches!(outcome, Err(Error::NegativePrice)), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn zero_price_accepted() -> Result<(), Error> {
        let p = Price::new(Decimal::ZERO)?;
        check(p.into_decimal() == Decimal::ZERO, || {
            format!("got {}", p.into_decimal())
        })
    }

    #[test]
    fn zero_quantity_rejected() -> Result<(), Error> {
        let outcome = Quantity::new(0);
        check(matches!(outcome, Err(Error::ZeroQuantity)), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn positive_quantity_accepted() -> Result<(), Error> {
        let q = Quantity::new(3)?;
        check(q.get() == 3, || format!("got {}", q.get()))
    }
}

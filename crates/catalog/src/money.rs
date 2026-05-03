//! [`Price`] value object backed by `rust_decimal::Decimal` for exact arithmetic.
//!
//! Catalog-side prices have no addition/subtraction semantics (an item
//! has one price, never a sum of prices), so [`Price`] is just a
//! non-negative wrapper around [`Decimal`].  The richer [`Money`] /
//! [`UnitPrice`] / [`Discount`] arithmetic lives in `ordering-domain`.

use core::fmt;

use rust_decimal::Decimal;

use crate::error::Error;

/// Per-item price.  Always non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(Decimal);

impl Price {
    /// Construct a non-negative [`Price`].
    ///
    /// # Errors
    /// Returns [`Error::NegativePrice`] if `value` is negative.
    pub fn new(value: Decimal) -> Result<Self, Error> {
        if value >= Decimal::ZERO {
            Ok(Self(value))
        } else {
            Err(Error::NegativePrice)
        }
    }

    /// Zero price.
    #[must_use]
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Underlying [`Decimal`].
    #[must_use]
    pub fn into_decimal(self) -> Decimal {
        self.0
    }
}

impl TryFrom<Decimal> for Price {
    type Error = Error;

    fn try_from(d: Decimal) -> Result<Self, Error> {
        Self::new(d)
    }
}

impl From<Price> for Decimal {
    fn from(p: Price) -> Self {
        p.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    #[test]
    fn zero_accepted() -> Result<(), Error> {
        let p = Price::new(Decimal::ZERO)?;
        check(p.into_decimal() == Decimal::ZERO, || {
            format!("round-trip mismatch: {p}")
        })
    }

    #[test]
    fn positive_accepted() -> Result<(), Error> {
        let p = Price::try_from(Decimal::new(1999, 2))?;
        check(p.into_decimal() == Decimal::new(1999, 2), || {
            format!("round-trip mismatch: {p}")
        })
    }

    #[test]
    fn negative_rejected() -> Result<(), Error> {
        let outcome = Price::new(Decimal::NEGATIVE_ONE);
        check(matches!(outcome, Err(Error::NegativePrice)), || {
            format!("expected NegativePrice, got {outcome:?}")
        })
    }
}

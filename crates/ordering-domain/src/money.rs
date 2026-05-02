//! [`Money`] value object backed by `rust_decimal::Decimal` for exact arithmetic.

use core::fmt;

use rust_decimal::Decimal;

use crate::error::Error;

/// A monetary amount.  Equality is exact on the underlying [`Decimal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(Decimal);

impl Money {
    /// Zero amount.
    #[must_use]
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Returns true if the amount is non-negative.
    #[must_use]
    pub fn is_non_negative(self) -> bool {
        self.0 >= Decimal::ZERO
    }

    /// Checked addition.
    ///
    /// # Errors
    /// Returns `Err(Error::DecimalOverflow)` on overflow.
    pub fn checked_add(self, rhs: Self) -> Result<Self, Error> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(Error::DecimalOverflow)
    }

    /// Checked subtraction.
    ///
    /// # Errors
    /// Returns `Err(Error::DecimalOverflow)` on overflow.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, Error> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(Error::DecimalOverflow)
    }

    /// Multiply by a unit count.
    ///
    /// # Errors
    /// Returns `Err(Error::DecimalOverflow)` on overflow.
    pub fn checked_mul_units(self, units: u32) -> Result<Self, Error> {
        self.0
            .checked_mul(Decimal::from(units))
            .map(Self)
            .ok_or(Error::DecimalOverflow)
    }
}

impl From<Decimal> for Money {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<Money> for Decimal {
    fn from(m: Money) -> Self {
        m.0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Per-unit price of a line item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitPrice(Money);

impl UnitPrice {
    /// Underlying [`Money`].
    #[must_use]
    pub fn money(self) -> Money {
        self.0
    }
}

impl From<Money> for UnitPrice {
    fn from(m: Money) -> Self {
        Self(m)
    }
}

impl From<UnitPrice> for Money {
    fn from(u: UnitPrice) -> Self {
        u.0
    }
}

/// Discount applied to a line item, in absolute money terms.  Always non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Discount(Money);

impl Discount {
    /// Construct a non-negative [`Discount`].
    ///
    /// # Errors
    /// Returns `Err(Error::NegativeDiscount)` if `m` is negative.
    pub fn new(m: Money) -> Result<Self, Error> {
        if m.is_non_negative() {
            Ok(Self(m))
        } else {
            Err(Error::NegativeDiscount)
        }
    }

    /// Zero discount.
    #[must_use]
    pub fn zero() -> Self {
        Self(Money::zero())
    }

    /// Underlying [`Money`].
    #[must_use]
    pub fn money(self) -> Money {
        self.0
    }
}

impl From<Discount> for Money {
    fn from(d: Discount) -> Self {
        d.0
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
    fn money_arithmetic_round_trip() -> Result<(), Error> {
        let a = Money::from(Decimal::from(10));
        let b = Money::from(Decimal::from(3));
        let sum = a.checked_add(b)?;
        let diff = sum.checked_sub(b)?;
        check(diff == a, || format!("({sum} - {b}) != {a}"))
    }

    #[test]
    fn money_mul_units() -> Result<(), Error> {
        let price = Money::from(Decimal::from(7));
        let total = price.checked_mul_units(3)?;
        check(total == Money::from(Decimal::from(21)), || {
            format!("expected 21, got {total}")
        })
    }

    #[test]
    fn discount_rejects_negative() -> Result<(), Error> {
        let neg = Money::from(Decimal::NEGATIVE_ONE);
        let outcome = Discount::new(neg);
        check(matches!(outcome, Err(Error::NegativeDiscount)), || {
            format!("expected NegativeDiscount, got {outcome:?}")
        })
    }
}

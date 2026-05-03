//! Stock-level newtypes for [`CatalogItem`](crate::item::CatalogItem).
//!
//! Upstream eShop stores `AvailableStock`, `RestockThreshold`, and
//! `MaxStockThreshold` as plain `int` columns; we wrap them in
//! [`Stock`], [`RestockThreshold`], and [`MaxStockThreshold`] newtypes
//! so the type system rejects mixing the three at call sites.
//!
//! [`Units`] is the quantity argument to
//! [`CatalogItem::add_stock`](crate::item::CatalogItem::add_stock) and
//! [`CatalogItem::remove_stock`](crate::item::CatalogItem::remove_stock);
//! it is a separate newtype because "how many to add" carries different
//! semantics from "how many we currently have".

/// Quantity currently in stock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stock(u32);

impl Stock {
    /// Underlying count.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }

    /// Returns true if there is no stock available.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for Stock {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl From<Stock> for u32 {
    fn from(s: Stock) -> Self {
        s.0
    }
}

/// Stock level at which a reorder is triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RestockThreshold(u32);

impl RestockThreshold {
    /// Underlying count.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for RestockThreshold {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl From<RestockThreshold> for u32 {
    fn from(r: RestockThreshold) -> Self {
        r.0
    }
}

/// Maximum stock level that the warehouse can hold for an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaxStockThreshold(u32);

impl MaxStockThreshold {
    /// Underlying count.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for MaxStockThreshold {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl From<MaxStockThreshold> for u32 {
    fn from(m: MaxStockThreshold) -> Self {
        m.0
    }
}

/// Quantity argument to
/// [`CatalogItem::add_stock`](crate::item::CatalogItem::add_stock) and
/// [`CatalogItem::remove_stock`](crate::item::CatalogItem::remove_stock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Units(u32);

impl Units {
    /// Underlying count.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }

    /// Returns true if `self` is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Returns the smaller of `self` and `other`.
    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl From<u32> for Units {
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl From<Units> for u32 {
    fn from(u: Units) -> Self {
        u.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    #[test]
    fn stock_zero_predicate() -> Result<(), Error> {
        check(Stock::from(0).is_zero(), || "0 not zero".to_string())?;
        check(!Stock::from(1).is_zero(), || "1 reported zero".to_string())
    }

    #[test]
    fn units_min_picks_smaller() -> Result<(), Error> {
        let a = Units::from(5);
        let b = Units::from(3);
        check(a.min(b).get() == 3, || {
            format!("expected 3, got {}", a.min(b).get())
        })
    }
}

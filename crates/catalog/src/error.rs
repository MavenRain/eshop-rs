//! Crate error type for the Catalog bounded context.
//!
//! Covers both domain validation (constructor invariants on
//! [`CatalogItem`](crate::item::CatalogItem) etc.) and the persistence
//! layer (toasty-backed repositories).  Domain-only callers can match on
//! the `EmptyString` / `NegativePrice` / `OutOfStock` /
//! `RestockExceedsMax` / `InitialStockExceedsMax` / `StockOverflow` /
//! `InvariantViolated` variants and ignore the rest; the persistence
//! variants only fire from the `row` / `mapper` / `repository` modules.

/// Errors produced by Catalog operations (domain + persistence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Construction of a non-empty string failed because the input was empty.
    EmptyString { field: &'static str },
    /// A [`Price`](crate::money::Price) was constructed with a negative amount.
    NegativePrice,
    /// [`CatalogItem::remove_stock`](crate::item::CatalogItem::remove_stock)
    /// was called when [`Stock`](crate::stock::Stock) was already zero.
    OutOfStock { item: String },
    /// `RestockThreshold` exceeded `MaxStockThreshold` at construction.
    RestockExceedsMax { restock: u32, max: u32 },
    /// Initial `Stock` exceeded `MaxStockThreshold` at construction.
    InitialStockExceedsMax { stock: u32, max: u32 },
    /// Stock arithmetic overflowed `u32`.
    StockOverflow,
    /// A domain invariant was violated; carries a human-readable reason.
    InvariantViolated { reason: String },
    /// Failure inside toasty.
    Toasty { reason: String },
    /// Numeric out-of-range for the target type when reading from a row.
    NumericRange { context: String },
    /// A persisted column held a value the loader could not interpret.
    InvalidPersistedValue { context: String, value: String },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyString { field } => write!(f, "{field} must not be empty"),
            Self::NegativePrice => write!(f, "price must not be negative"),
            Self::OutOfStock { item } => {
                write!(f, "empty stock, product item {item} is sold out")
            }
            Self::RestockExceedsMax { restock, max } => write!(
                f,
                "restock threshold {restock} exceeds max stock threshold {max}"
            ),
            Self::InitialStockExceedsMax { stock, max } => {
                write!(f, "initial stock {stock} exceeds max stock threshold {max}")
            }
            Self::StockOverflow => write!(f, "stock arithmetic overflowed"),
            Self::InvariantViolated { reason } => write!(f, "invariant violated: {reason}"),
            Self::Toasty { reason } => write!(f, "toasty error: {reason}"),
            Self::NumericRange { context } => write!(f, "numeric out of range at {context}"),
            Self::InvalidPersistedValue { context, value } => {
                write!(f, "invalid persisted value at {context}: {value}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<toasty::Error> for Error {
    fn from(e: toasty::Error) -> Self {
        Self::Toasty {
            reason: e.to_string(),
        }
    }
}

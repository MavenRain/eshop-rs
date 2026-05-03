//! Domain error type for the Catalog bounded context.

/// Errors produced by Catalog domain operations.
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
        }
    }
}

impl std::error::Error for Error {}

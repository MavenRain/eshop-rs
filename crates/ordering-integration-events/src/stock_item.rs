//! Wire-form representation of an order line item.
//!
//! Mirrors upstream eShop's `OrderStockItem` (`Catalog.IntegrationEvents`).
//! Carries plain primitives so the wire format is independent of the
//! `ordering-domain` newtypes.

use serde::{Deserialize, Serialize};

/// Product id and ordered units, ready to be serialized into an
/// integration event payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderStockItem {
    product_id: i32,
    units: u32,
}

impl OrderStockItem {
    /// Construct.
    #[must_use]
    pub fn new(product_id: i32, units: u32) -> Self {
        Self { product_id, units }
    }

    /// Catalog product identifier.
    #[must_use]
    pub fn product_id(&self) -> i32 {
        self.product_id
    }

    /// Ordered units.
    #[must_use]
    pub fn units(&self) -> u32 {
        self.units
    }
}

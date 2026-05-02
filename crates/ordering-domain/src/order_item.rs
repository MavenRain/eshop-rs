//! [`OrderItem`] entity within the [`Order`](crate::order::Order) aggregate.

use core::num::NonZeroU32;

use uuid::Uuid;

use crate::error::Error;
use crate::money::{Discount, Money, UnitPrice};
use crate::strings::{PictureUrl, ProductName};

/// Identifier for an [`OrderItem`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct OrderItemId(Uuid);

impl OrderItemId {
    /// Generate a fresh identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Underlying [`Uuid`].
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for OrderItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for OrderItemId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<OrderItemId> for Uuid {
    fn from(id: OrderItemId) -> Self {
        id.0
    }
}

/// Catalog product identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ProductId(i32);

impl From<i32> for ProductId {
    fn from(n: i32) -> Self {
        Self(n)
    }
}

impl From<ProductId> for i32 {
    fn from(p: ProductId) -> Self {
        p.0
    }
}

/// Quantity of a [`ProductId`] in a line item.  Always positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Units(NonZeroU32);

impl Units {
    /// Construct from a `u32`.
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidUnits)` if `n` is zero.
    pub fn new(n: u32) -> Result<Self, Error> {
        NonZeroU32::new(n).map(Self).ok_or(Error::InvalidUnits)
    }

    /// Underlying `u32`.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl From<NonZeroU32> for Units {
    fn from(n: NonZeroU32) -> Self {
        Self(n)
    }
}

impl From<Units> for u32 {
    fn from(u: Units) -> Self {
        u.0.get()
    }
}

/// A line item in an [`Order`](crate::order::Order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    id: OrderItemId,
    product_id: ProductId,
    product_name: ProductName,
    picture_url: PictureUrl,
    unit_price: UnitPrice,
    discount: Discount,
    units: Units,
}

impl OrderItem {
    /// Construct an [`OrderItem`] enforcing `unit_price * units >= discount`.
    ///
    /// # Errors
    /// Returns `Err(Error::DiscountExceedsTotal)` if the line total is less
    /// than the discount.  Returns `Err(Error::DecimalOverflow)` on
    /// arithmetic overflow.
    pub fn new(
        id: OrderItemId,
        product_id: ProductId,
        product_name: ProductName,
        picture_url: PictureUrl,
        unit_price: UnitPrice,
        discount: Discount,
        units: Units,
    ) -> Result<Self, Error> {
        let total = unit_price.money().checked_mul_units(units.get())?;
        if Money::from(discount) > total {
            Err(Error::DiscountExceedsTotal)
        } else {
            Ok(Self {
                id,
                product_id,
                product_name,
                picture_url,
                unit_price,
                discount,
                units,
            })
        }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> OrderItemId {
        self.id
    }

    /// Product identifier.
    #[must_use]
    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    /// Product name.
    #[must_use]
    pub fn product_name(&self) -> &ProductName {
        &self.product_name
    }

    /// Picture URL.
    #[must_use]
    pub fn picture_url(&self) -> &PictureUrl {
        &self.picture_url
    }

    /// Unit price.
    #[must_use]
    pub fn unit_price(&self) -> UnitPrice {
        self.unit_price
    }

    /// Discount.
    #[must_use]
    pub fn discount(&self) -> Discount {
        self.discount
    }

    /// Units.
    #[must_use]
    pub fn units(&self) -> Units {
        self.units
    }

    /// Line total: `unit_price * units`.
    ///
    /// # Errors
    /// Returns `Err(Error::DecimalOverflow)` on arithmetic overflow.
    pub fn line_total(&self) -> Result<Money, Error> {
        self.unit_price.money().checked_mul_units(self.units.get())
    }

    /// Add `extra` units, returning a new [`OrderItem`].
    ///
    /// # Errors
    /// Returns `Err(Error::InvalidUnits)` if the resulting unit count
    /// overflows `u32`.  Returns `Err(Error::DiscountExceedsTotal)` only if
    /// the existing discount somehow exceeds the new total (unlikely since
    /// the total grows).  Returns `Err(Error::DecimalOverflow)` on overflow.
    pub fn add_units(self, extra: Units) -> Result<Self, Error> {
        let combined = self
            .units
            .get()
            .checked_add(extra.get())
            .and_then(NonZeroU32::new)
            .ok_or(Error::InvalidUnits)?;
        Self::new(
            self.id,
            self.product_id,
            self.product_name,
            self.picture_url,
            self.unit_price,
            self.discount,
            Units::from(combined),
        )
    }

    /// Replace the discount if `candidate` exceeds the current discount.
    ///
    /// # Errors
    /// Returns `Err(Error::DiscountExceedsTotal)` if the new discount would
    /// exceed the line total.  Returns `Err(Error::DecimalOverflow)` on
    /// arithmetic overflow.
    pub fn set_max_discount(self, candidate: Discount) -> Result<Self, Error> {
        if candidate > self.discount {
            Self::new(
                self.id,
                self.product_id,
                self.product_name,
                self.picture_url,
                self.unit_price,
                candidate,
                self.units,
            )
        } else {
            Ok(self)
        }
    }
}

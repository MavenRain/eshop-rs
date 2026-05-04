//! [`BasketItem`] aggregate-internal value type and its
//! [`BasketItemId`] newtype.

use uuid::Uuid;

use crate::money::{Price, Quantity};
use crate::product::ProductId;
use crate::strings::{PictureUrl, ProductName};

/// Identifier for a single line item within a [`CustomerBasket`](crate::CustomerBasket).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasketItemId(Uuid);

impl BasketItemId {
    /// Generate a fresh random id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow as a [`Uuid`] (used by mappers and DTOs).
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for BasketItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BasketItemId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<BasketItemId> for Uuid {
    fn from(id: BasketItemId) -> Self {
        id.0
    }
}

/// Snapshot of a catalog product as it sits inside a customer's
/// basket: product reference, denormalized name and picture, the
/// price that was current when the item was added (plus the price
/// before, when the catalog later changed it), and the quantity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasketItem {
    id: BasketItemId,
    product_id: ProductId,
    product_name: ProductName,
    unit_price: Price,
    old_unit_price: Option<Price>,
    quantity: Quantity,
    picture_url: Option<PictureUrl>,
}

impl BasketItem {
    /// Construct.
    ///
    /// `old_unit_price` is `Some` when the catalog's price for this
    /// product has changed since the item was added (the UI uses
    /// it to show a strike-through).
    #[must_use]
    pub fn new(
        id: BasketItemId,
        product_id: ProductId,
        product_name: ProductName,
        unit_price: Price,
        old_unit_price: Option<Price>,
        quantity: Quantity,
        picture_url: Option<PictureUrl>,
    ) -> Self {
        Self {
            id,
            product_id,
            product_name,
            unit_price,
            old_unit_price,
            quantity,
            picture_url,
        }
    }

    /// Item identifier.
    #[must_use]
    pub fn id(&self) -> BasketItemId {
        self.id
    }

    /// Catalog product id.
    #[must_use]
    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    /// Snapshotted product name.
    #[must_use]
    pub fn product_name(&self) -> &ProductName {
        &self.product_name
    }

    /// Current unit price.
    #[must_use]
    pub fn unit_price(&self) -> Price {
        self.unit_price
    }

    /// Previous unit price, if the catalog's price has changed since
    /// the item entered the basket.
    #[must_use]
    pub fn old_unit_price(&self) -> Option<Price> {
        self.old_unit_price
    }

    /// Quantity of this product in the basket.
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Optional picture url for the basket UI.
    #[must_use]
    pub fn picture_url(&self) -> Option<&PictureUrl> {
        self.picture_url.as_ref()
    }
}

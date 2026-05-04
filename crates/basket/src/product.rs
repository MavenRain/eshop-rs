//! [`ProductId`] newtype, matching upstream eShop's catalog product id
//! width.
//!
//! Upstream uses `int` for `Catalog.API`'s `Id` column and the same
//! `int` flows through `BasketItem.ProductId` and ordering's
//! `OrderItem.ProductId`.  Our ordering crate already aligns with
//! that shape (`ordering_domain::ProductId` is `i32`); basket
//! follows suit so checkout can hand items off to ordering without
//! a translation layer.

/// Catalog product identifier as carried inside a basket item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProductId(i32);

impl From<i32> for ProductId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<ProductId> for i32 {
    fn from(id: ProductId) -> Self {
        id.0
    }
}

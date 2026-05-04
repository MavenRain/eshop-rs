//! [`CustomerId`] newtype.

use uuid::Uuid;

/// Identifier for the customer who owns a basket.
///
/// Upstream eShop's `BuyerId` is a free-form string; we narrow to
/// [`Uuid`] so basket and ordering can both speak the same identifier
/// shape (ordering's [`BuyerId`](https://docs.rs/ordering-domain) is
/// also a Uuid newtype).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomerId(Uuid);

impl CustomerId {
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

impl Default for CustomerId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for CustomerId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<CustomerId> for Uuid {
    fn from(id: CustomerId) -> Self {
        id.0
    }
}

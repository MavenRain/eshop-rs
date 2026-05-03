//! [`CatalogBrand`] aggregate.
//!
//! Mirrors `eShop.Catalog.API.Model.CatalogBrand` (`int Id`, `string
//! Brand`).  We use a [`Uuid`] newtype id instead of an int identity
//! column for parity with `ordering-domain`'s id treatment.

use uuid::Uuid;

use crate::strings::BrandName;

/// Identifier for a [`CatalogBrand`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CatalogBrandId(Uuid);

impl CatalogBrandId {
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

impl Default for CatalogBrandId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for CatalogBrandId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<CatalogBrandId> for Uuid {
    fn from(id: CatalogBrandId) -> Self {
        id.0
    }
}

/// Brand under which a [`CatalogItem`](crate::item::CatalogItem) is sold.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogBrand {
    id: CatalogBrandId,
    name: BrandName,
}

impl CatalogBrand {
    /// Construct.
    #[must_use]
    pub fn new(id: CatalogBrandId, name: BrandName) -> Self {
        Self { id, name }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> CatalogBrandId {
        self.id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &BrandName {
        &self.name
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
    fn brand_id_round_trip() -> Result<(), Error> {
        let raw = Uuid::new_v4();
        let id = CatalogBrandId::from(raw);
        check(id.into_uuid() == raw, || "round-trip mismatch".to_string())
    }

    #[test]
    fn brand_accessors() -> Result<(), Error> {
        let id = CatalogBrandId::new();
        let name = BrandName::try_from(".NET")?;
        let brand = CatalogBrand::new(id, name.clone());
        check(brand.id() == id, || "id mismatch".to_string())?;
        check(brand.name() == &name, || "name mismatch".to_string())
    }
}

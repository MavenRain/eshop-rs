//! [`CatalogKind`] aggregate (upstream's `CatalogType`).
//!
//! Renamed from upstream's `CatalogType` because `type` is a reserved
//! keyword in Rust.  "Kind" is the standard alternative when you want
//! to mean a category or classification ("`Mug`", "`T-Shirt`", "`USB
//! Memory Stick`" are kinds of catalog items).

use uuid::Uuid;

use crate::strings::KindName;

/// Identifier for a [`CatalogKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CatalogKindId(Uuid);

impl CatalogKindId {
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

impl Default for CatalogKindId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for CatalogKindId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<CatalogKindId> for Uuid {
    fn from(id: CatalogKindId) -> Self {
        id.0
    }
}

/// Category under which a [`CatalogItem`](crate::item::CatalogItem) is
/// classified (upstream `CatalogType`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogKind {
    id: CatalogKindId,
    name: KindName,
}

impl CatalogKind {
    /// Construct.
    #[must_use]
    pub fn new(id: CatalogKindId, name: KindName) -> Self {
        Self { id, name }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> CatalogKindId {
        self.id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &KindName {
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
    fn kind_accessors() -> Result<(), Error> {
        let id = CatalogKindId::new();
        let name = KindName::try_from("Mug")?;
        let kind = CatalogKind::new(id, name.clone());
        check(kind.id() == id, || "id mismatch".to_string())?;
        check(kind.name() == &name, || "name mismatch".to_string())
    }
}

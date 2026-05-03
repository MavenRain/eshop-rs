//! Catalog-specific non-empty string newtypes.
//!
//! Each newtype wraps `String` and rejects whitespace-only or empty input
//! at construction via `TryFrom<String>` / `TryFrom<&str>`.

use crate::error::Error;

macro_rules! non_empty_string_newtype {
    ($(#[$meta:meta])* $name:ident, $field:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Borrow the inner string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(s: String) -> Result<Self, Error> {
                if s.trim().is_empty() {
                    Err(Error::EmptyString { field: $field })
                } else {
                    Ok(Self(s))
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(s: &str) -> Result<Self, Error> {
                Self::try_from(s.to_string())
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.0
            }
        }
    };
}

non_empty_string_newtype!(
    /// Display name of a [`CatalogItem`](crate::item::CatalogItem).
    ItemName,
    "item name"
);
non_empty_string_newtype!(
    /// Display name of a [`CatalogBrand`](crate::brand::CatalogBrand).
    BrandName,
    "brand name"
);
non_empty_string_newtype!(
    /// Display name of a [`CatalogKind`](crate::kind::CatalogKind).
    KindName,
    "kind name"
);
non_empty_string_newtype!(
    /// Description text for a [`CatalogItem`](crate::item::CatalogItem).
    ///
    /// Items without a description are modelled as
    /// `Option<ItemDescription>`; an item with `Some(_)` here always has a
    /// non-empty description string.
    ItemDescription,
    "item description"
);
non_empty_string_newtype!(
    /// Picture file name for a [`CatalogItem`](crate::item::CatalogItem).
    ///
    /// Modelled as `Option<PictureFileName>`; if present, always non-empty.
    PictureFileName,
    "picture file name"
);

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::InvariantViolated { reason: reason() })
        }
    }

    #[test]
    fn empty_string_rejected() -> Result<(), Error> {
        let outcome = ItemName::try_from("");
        check(
            matches!(outcome, Err(Error::EmptyString { field: "item name" })),
            || format!("expected EmptyString for empty input, got {outcome:?}"),
        )
    }

    #[test]
    fn whitespace_only_rejected() -> Result<(), Error> {
        let outcome = BrandName::try_from("   ");
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "brand name"
                })
            ),
            || format!("expected EmptyString for whitespace, got {outcome:?}"),
        )
    }

    #[test]
    fn non_empty_accepted() -> Result<(), Error> {
        let s = ItemName::try_from(".NET Bot Black Hoodie")?;
        check(s.as_str() == ".NET Bot Black Hoodie", || {
            format!("round-trip mismatch: {}", s.as_str())
        })
    }

    #[test]
    fn round_trip_into_string() -> Result<(), Error> {
        let n = KindName::try_from("Mug")?;
        let raw: String = n.into();
        check(raw == "Mug", || format!("round-trip mismatch: {raw}"))
    }
}

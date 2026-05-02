//! Domain-specific non-empty string newtypes.
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
    /// Street component of an [`Address`](crate::address::Address).
    Street,
    "street"
);
non_empty_string_newtype!(
    /// City component of an [`Address`](crate::address::Address).
    City,
    "city"
);
non_empty_string_newtype!(
    /// State or region component of an [`Address`](crate::address::Address).
    State,
    "state"
);
non_empty_string_newtype!(
    /// Country component of an [`Address`](crate::address::Address).
    Country,
    "country"
);
non_empty_string_newtype!(
    /// Zip or postal code component of an [`Address`](crate::address::Address).
    ZipCode,
    "zip code"
);
non_empty_string_newtype!(
    /// External identity provider's user identifier.
    UserId,
    "user id"
);
non_empty_string_newtype!(
    /// Display name of a user.
    UserName,
    "user name"
);
non_empty_string_newtype!(
    /// External identity GUID for a [`Buyer`](crate::buyer::Buyer).
    IdentityGuid,
    "identity guid"
);
non_empty_string_newtype!(
    /// Catalog product display name.
    ProductName,
    "product name"
);
non_empty_string_newtype!(
    /// URL of a product picture.
    PictureUrl,
    "picture url"
);
non_empty_string_newtype!(
    /// Card number on a [`PaymentMethod`](crate::payment_method::PaymentMethod).
    CardNumber,
    "card number"
);
non_empty_string_newtype!(
    /// Card security code (CVV).
    SecurityNumber,
    "security number"
);
non_empty_string_newtype!(
    /// Cardholder name on a [`PaymentMethod`](crate::payment_method::PaymentMethod).
    CardHolderName,
    "card holder name"
);
non_empty_string_newtype!(
    /// Human-readable alias for a [`PaymentMethod`](crate::payment_method::PaymentMethod).
    CardAlias,
    "card alias"
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
        let outcome = Street::try_from("");
        check(
            matches!(outcome, Err(Error::EmptyString { field: "street" })),
            || format!("expected EmptyString for empty input, got {outcome:?}"),
        )
    }

    #[test]
    fn whitespace_only_rejected() -> Result<(), Error> {
        let outcome = Street::try_from("   ");
        check(
            matches!(outcome, Err(Error::EmptyString { field: "street" })),
            || format!("expected EmptyString for whitespace, got {outcome:?}"),
        )
    }

    #[test]
    fn non_empty_accepted() -> Result<(), Error> {
        let s = Street::try_from("100 Main St")?;
        check(s.as_str() == "100 Main St", || {
            format!("round-trip mismatch: {}", s.as_str())
        })
    }
}

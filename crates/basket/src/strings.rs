//! Non-empty string newtypes for basket payload fields.

use crate::error::Error;

/// Display name of a catalog product, snapshotted into the basket
/// item.  Required, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductName(String);

impl ProductName {
    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProductName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        if value.trim().is_empty() {
            Err(Error::EmptyString {
                field: "product name",
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ProductName {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        Self::try_from(value.to_string())
    }
}

/// URL-shaped pointer to the product's display image.  Optional
/// at the basket-item level (upstream allows it to be absent),
/// but when present, must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PictureUrl(String);

impl PictureUrl {
    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PictureUrl {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        if value.trim().is_empty() {
            Err(Error::EmptyString {
                field: "picture url",
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for PictureUrl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        Self::try_from(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(cond: bool, reason: impl FnOnce() -> String) -> Result<(), Error> {
        if cond {
            Ok(())
        } else {
            Err(Error::Validation { reason: reason() })
        }
    }

    #[test]
    fn empty_product_name_rejected() -> Result<(), Error> {
        let outcome = ProductName::try_from(String::new());
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "product name"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn whitespace_only_product_name_rejected() -> Result<(), Error> {
        let outcome = ProductName::try_from("   ");
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "product name"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn non_empty_product_name_accepted() -> Result<(), Error> {
        let name = ProductName::try_from(".NET Bot Hoodie")?;
        check(name.as_str() == ".NET Bot Hoodie", || {
            format!("got {}", name.as_str())
        })
    }

    #[test]
    fn empty_picture_url_rejected() -> Result<(), Error> {
        let outcome = PictureUrl::try_from(String::new());
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "picture url"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }
}

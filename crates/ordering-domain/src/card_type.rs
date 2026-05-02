//! Card brand.

use core::fmt;

use crate::error::Error;

/// Card brand.  The numeric IDs in [`CardType::id`] mirror the upstream
/// `dotnet/eShop` reference rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    /// American Express.
    Amex,
    /// Visa.
    Visa,
    /// `MasterCard`.
    MasterCard,
}

impl CardType {
    /// Numeric ID of a card brand.
    #[must_use]
    pub fn id(self) -> i32 {
        match self {
            Self::Amex => 1,
            Self::Visa => 2,
            Self::MasterCard => 3,
        }
    }

    /// Display name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Amex => "Amex",
            Self::Visa => "Visa",
            Self::MasterCard => "MasterCard",
        }
    }
}

impl TryFrom<i32> for CardType {
    type Error = Error;

    fn try_from(id: i32) -> Result<Self, Error> {
        match id {
            1 => Ok(Self::Amex),
            2 => Ok(Self::Visa),
            3 => Ok(Self::MasterCard),
            other => Err(Error::UnknownCardType { id: other }),
        }
    }
}

impl From<CardType> for i32 {
    fn from(c: CardType) -> Self {
        c.id()
    }
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

//! [`PaymentMethod`] entity within the [`Buyer`](crate::buyer::Buyer) aggregate.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::card_type::CardType;
use crate::error::Error;
use crate::strings::{CardAlias, CardHolderName, CardNumber, SecurityNumber};

/// Identifier for a [`PaymentMethod`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PaymentMethodId(Uuid);

impl PaymentMethodId {
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

impl Default for PaymentMethodId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for PaymentMethodId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<PaymentMethodId> for Uuid {
    fn from(id: PaymentMethodId) -> Self {
        id.0
    }
}

/// Card expiration timestamp.  Validated to be strictly in the future at
/// construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CardExpiration(DateTime<Utc>);

impl CardExpiration {
    /// Construct, requiring `expiration > now`.
    ///
    /// # Errors
    /// Returns `Err(Error::CardExpired)` if `expiration <= now`.
    pub fn new(expiration: DateTime<Utc>, now: DateTime<Utc>) -> Result<Self, Error> {
        if expiration > now {
            Ok(Self(expiration))
        } else {
            Err(Error::CardExpired)
        }
    }

    /// Underlying timestamp.
    #[must_use]
    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }
}

impl From<CardExpiration> for DateTime<Utc> {
    fn from(e: CardExpiration) -> Self {
        e.0
    }
}

/// A payment method belonging to a [`Buyer`](crate::buyer::Buyer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentMethod {
    id: PaymentMethodId,
    alias: CardAlias,
    card_number: CardNumber,
    security_number: SecurityNumber,
    card_holder_name: CardHolderName,
    expiration: CardExpiration,
    card_type: CardType,
}

impl PaymentMethod {
    /// Construct.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        id: PaymentMethodId,
        alias: CardAlias,
        card_number: CardNumber,
        security_number: SecurityNumber,
        card_holder_name: CardHolderName,
        expiration: CardExpiration,
        card_type: CardType,
    ) -> Self {
        Self {
            id,
            alias,
            card_number,
            security_number,
            card_holder_name,
            expiration,
            card_type,
        }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> PaymentMethodId {
        self.id
    }

    /// Alias.
    #[must_use]
    pub fn alias(&self) -> &CardAlias {
        &self.alias
    }

    /// Card number.  Treat as PCI-sensitive at infrastructure boundaries.
    #[must_use]
    pub fn card_number(&self) -> &CardNumber {
        &self.card_number
    }

    /// Card security code.  Treat as PCI-sensitive at infrastructure boundaries.
    #[must_use]
    pub fn security_number(&self) -> &SecurityNumber {
        &self.security_number
    }

    /// Cardholder name.
    #[must_use]
    pub fn card_holder_name(&self) -> &CardHolderName {
        &self.card_holder_name
    }

    /// Card expiration.
    #[must_use]
    pub fn expiration(&self) -> CardExpiration {
        self.expiration
    }

    /// Card brand.
    #[must_use]
    pub fn card_type(&self) -> CardType {
        self.card_type
    }

    /// Whether this payment method matches `(card_type, card_number, expiration)`.
    #[must_use]
    pub fn is_equal_to(
        &self,
        card_type: CardType,
        card_number: &CardNumber,
        expiration: CardExpiration,
    ) -> bool {
        self.card_type == card_type
            && self.card_number == *card_number
            && self.expiration == expiration
    }
}

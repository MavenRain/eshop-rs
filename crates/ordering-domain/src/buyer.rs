//! [`Buyer`] aggregate root.

use uuid::Uuid;

use crate::payment_method::PaymentMethod;
use crate::strings::{IdentityGuid, UserName};

/// Identifier for a [`Buyer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BuyerId(Uuid);

impl BuyerId {
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

impl Default for BuyerId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BuyerId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<BuyerId> for Uuid {
    fn from(id: BuyerId) -> Self {
        id.0
    }
}

/// Aggregate root representing the customer placing orders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buyer {
    id: BuyerId,
    identity_guid: IdentityGuid,
    name: UserName,
    payment_methods: Vec<PaymentMethod>,
}

impl Buyer {
    /// Construct a new [`Buyer`] with no payment methods.
    #[must_use]
    pub fn new(id: BuyerId, identity_guid: IdentityGuid, name: UserName) -> Self {
        Self {
            id,
            identity_guid,
            name,
            payment_methods: Vec::new(),
        }
    }

    /// Identifier.
    #[must_use]
    pub fn id(&self) -> BuyerId {
        self.id
    }

    /// External identity GUID.
    #[must_use]
    pub fn identity_guid(&self) -> &IdentityGuid {
        &self.identity_guid
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &UserName {
        &self.name
    }

    /// Read-only view of registered payment methods.
    #[must_use]
    pub fn payment_methods(&self) -> &[PaymentMethod] {
        &self.payment_methods
    }

    /// Add a payment method, returning the new [`Buyer`].
    #[must_use]
    pub fn with_payment_method(self, payment: PaymentMethod) -> Self {
        let payment_methods = self
            .payment_methods
            .into_iter()
            .chain(core::iter::once(payment))
            .collect();
        Self {
            payment_methods,
            ..self
        }
    }
}

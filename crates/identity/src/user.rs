//! [`User`] aggregate.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::strings::{Email, PasswordHash, UserName};

/// Identifier for a [`User`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct UserId(Uuid);

impl UserId {
    /// Generate a fresh random identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow as the underlying [`Uuid`].
    #[must_use]
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

/// One user account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    email: Email,
    name: UserName,
    password_hash: PasswordHash,
    created_at: DateTime<Utc>,
}

impl User {
    /// Construct.
    #[must_use]
    pub fn new(
        id: UserId,
        email: Email,
        name: UserName,
        password_hash: PasswordHash,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            password_hash,
            created_at,
        }
    }

    /// User identifier.
    #[must_use]
    pub fn id(&self) -> UserId {
        self.id
    }

    /// Login email.
    #[must_use]
    pub fn email(&self) -> &Email {
        &self.email
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &UserName {
        &self.name
    }

    /// Stored password hash.
    #[must_use]
    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    /// Creation timestamp (UTC).
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

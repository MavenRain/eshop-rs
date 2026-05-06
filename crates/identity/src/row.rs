//! Toasty row for [`User`](crate::user::User).

use jiff::Timestamp;
use uuid::Uuid;

/// One row per user account.
#[derive(Debug, toasty::Model)]
pub struct UserRow {
    /// User identifier (primary key).
    #[key]
    pub id: Uuid,

    /// Login email.  Indexed so the login flow can look users up by
    /// email without scanning.
    #[index]
    pub email: String,

    /// Display name.
    pub name: String,

    /// Argon2 PHC string.
    pub password_hash: String,

    /// Creation timestamp.
    pub created_at: Timestamp,
}

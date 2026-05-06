//! Domain-from-row mapper.
//!
//! The reverse direction (`User` → row columns) lives inline in
//! `crate::repository::UserRepository::add`: there is no intermediate
//! "bag of columns" helper struct.  The `TryFrom<&UserRow>` impl
//! below is the only persistence boundary mapping the rest of the
//! crate consumes.

use crate::error::Error;
use crate::row::UserRow;
use crate::strings::{Email, PasswordHash, UserName};
use crate::time::jiff_to_chrono;
use crate::user::{User, UserId};

impl TryFrom<&UserRow> for User {
    type Error = Error;

    /// Rehydrate a [`User`] from a loaded row.
    ///
    /// Errors:
    /// - [`Error::TimeConversion`] if `created_at` is out of chrono's
    ///   representable range.
    /// - [`Error::EmptyString`] / [`Error::MalformedEmail`] if a
    ///   non-empty newtype was persisted blank or malformed
    ///   (database integrity break).
    fn try_from(row: &UserRow) -> Result<Self, Error> {
        Ok(User::new(
            UserId::from(row.id),
            Email::try_from(row.email.clone())?,
            UserName::try_from(row.name.clone())?,
            PasswordHash::try_from(row.password_hash.clone())?,
            jiff_to_chrono(row.created_at)?,
        ))
    }
}

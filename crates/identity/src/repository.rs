//! Persistence boundary for the [`User`] aggregate.

use crate::error::Error;
use crate::row::UserRow;
use crate::time::chrono_to_jiff;
use crate::user::{User, UserId};

/// Persistence boundary for [`User`].
pub struct UserRepository;

impl UserRepository {
    /// Insert a new user.
    ///
    /// # Errors
    /// Propagates toasty errors and time-conversion errors.
    pub async fn add<E>(executor: &mut E, user: &User) -> Result<(), Error>
    where
        E: toasty::Executor,
    {
        toasty::create!(UserRow {
            id: user.id().into_uuid(),
            email: user.email().as_str().to_string(),
            name: user.name().as_str().to_string(),
            password_hash: user.password_hash().as_str().to_string(),
            created_at: chrono_to_jiff(user.created_at())?,
        })
        .exec(executor)
        .await?;
        Ok(())
    }

    /// Load a user by id.
    ///
    /// # Errors
    /// Propagates toasty errors (including not-found) and mapping
    /// errors.
    pub async fn get_by_id<E>(executor: &mut E, id: UserId) -> Result<User, Error>
    where
        E: toasty::Executor,
    {
        let row = UserRow::get_by_id(executor, &id.into_uuid()).await?;
        User::try_from(&row)
    }

    /// Look up a user by email.  Returns the first match, or
    /// [`Error::NotFound`] if no user has that email.
    ///
    /// # Errors
    /// Propagates toasty errors and mapping errors.
    pub async fn find_by_email<E>(executor: &mut E, email: &str) -> Result<User, Error>
    where
        E: toasty::Executor,
    {
        let needle = email.to_string();
        let rows: Vec<UserRow> = toasty::query!(
            UserRow FILTER .email == #needle
        )
        .exec(executor)
        .await?;
        rows.first()
            .ok_or_else(|| Error::NotFound {
                reason: format!("no user with email {email}"),
            })
            .and_then(User::try_from)
    }
}

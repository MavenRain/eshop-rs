//! Newtypes for the Identity bounded context.

use crate::error::Error;

/// User-supplied display name.  Required, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserName(String);

impl UserName {
    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for UserName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        if value.trim().is_empty() {
            Err(Error::EmptyString { field: "user name" })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for UserName {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        Self::try_from(value.to_string())
    }
}

/// Email newtype.  Required, non-empty, and must contain `@`
/// (minimal shape check; full RFC 5322 validation is out of scope
/// for v1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Email {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        if value.trim().is_empty() {
            Err(Error::EmptyString { field: "email" })
        } else if !value.contains('@') {
            Err(Error::MalformedEmail { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for Email {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        Self::try_from(value.to_string())
    }
}

/// Argon2 password hash, as a serialized PHC string.  The
/// constructor is permissive (any non-empty string); validation
/// happens by attempting to parse via [`PasswordHash::new`](argon2::PasswordHash::new)
/// at compare time, in `crate::credential`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PasswordHash {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        if value.trim().is_empty() {
            Err(Error::EmptyString {
                field: "password hash",
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for PasswordHash {
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
    fn email_requires_at_sign() -> Result<(), Error> {
        let outcome = Email::try_from("alice");
        check(matches!(outcome, Err(Error::MalformedEmail { .. })), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn empty_email_rejected() -> Result<(), Error> {
        let outcome = Email::try_from(String::new());
        check(
            matches!(outcome, Err(Error::EmptyString { field: "email" })),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn well_formed_email_accepted() -> Result<(), Error> {
        let email = Email::try_from("alice@example.test")?;
        check(email.as_str() == "alice@example.test", || {
            format!("got {}", email.as_str())
        })
    }

    #[test]
    fn empty_user_name_rejected() -> Result<(), Error> {
        let outcome = UserName::try_from("   ");
        check(
            matches!(outcome, Err(Error::EmptyString { field: "user name" })),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn empty_password_hash_rejected() -> Result<(), Error> {
        let outcome = PasswordHash::try_from(String::new());
        check(
            matches!(
                outcome,
                Err(Error::EmptyString {
                    field: "password hash"
                })
            ),
            || format!("got {outcome:?}"),
        )
    }
}

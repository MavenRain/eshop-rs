//! Request DTOs deserialized from JSON bodies.
//!
//! Fields are private; serde derives expand in this module and have
//! access to them.  Public projection methods convert each request
//! into the corresponding domain values, validating along the way.

use serde::Deserialize;

use crate::error::Error;
use crate::strings::{Email, UserName};

/// `POST /api/identity/register` body.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    email: String,
    name: String,
    /// Plaintext.  Hashed by the handler before persistence.
    password: String,
}

impl RegisterRequest {
    /// Project into a `(Email, UserName, password_plaintext)` triple
    /// the registration handler can hand to the credential and
    /// persistence layers.
    ///
    /// # Errors
    /// - [`Error::EmptyString`] for blank `name` or `password`.
    /// - [`Error::MalformedEmail`] for an `email` lacking `@`.
    pub fn try_split(self) -> Result<(Email, UserName, String), Error> {
        require_non_empty(&self.password, "password")?;
        let email = Email::try_from(self.email)?;
        let name = UserName::try_from(self.name)?;
        Ok((email, name, self.password))
    }
}

/// `POST /api/identity/login` body.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

impl LoginRequest {
    /// Project into a `(Email, password_plaintext)` pair.
    ///
    /// # Errors
    /// - [`Error::EmptyString`] for blank `password`.
    /// - [`Error::MalformedEmail`] for an `email` lacking `@`.
    pub fn try_split(self) -> Result<(Email, String), Error> {
        require_non_empty(&self.password, "password")?;
        let email = Email::try_from(self.email)?;
        Ok((email, self.password))
    }
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), Error> {
    if value.trim().is_empty() {
        Err(Error::EmptyString { field })
    } else {
        Ok(())
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
    fn register_rejects_empty_password() -> Result<(), Error> {
        let request = RegisterRequest {
            email: "alice@example.test".to_string(),
            name: "Alice".to_string(),
            password: "   ".to_string(),
        };
        let outcome = request.try_split();
        check(
            matches!(outcome, Err(Error::EmptyString { field: "password" })),
            || format!("got {outcome:?}"),
        )
    }

    #[test]
    fn register_rejects_malformed_email() -> Result<(), Error> {
        let request = RegisterRequest {
            email: "alice".to_string(),
            name: "Alice".to_string(),
            password: "hunter2".to_string(),
        };
        let outcome = request.try_split();
        check(matches!(outcome, Err(Error::MalformedEmail { .. })), || {
            format!("got {outcome:?}")
        })
    }

    #[test]
    fn login_projects_to_email_and_password() -> Result<(), Error> {
        let request = LoginRequest {
            email: "alice@example.test".to_string(),
            password: "hunter2".to_string(),
        };
        let (email, password) = request.try_split()?;
        check(email.as_str() == "alice@example.test", || {
            format!("email {}", email.as_str())
        })?;
        check(password == "hunter2", || format!("password {password}"))
    }
}

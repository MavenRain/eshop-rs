//! Password hashing and verification, on top of the `argon2` crate.

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

use crate::error::Error;
use crate::strings::PasswordHash as StoredHash;

/// Hash a plaintext password with argon2 + a fresh random salt.
/// Returns the serialized PHC string suitable for storage.
///
/// # Errors
/// Returns [`Error::PasswordHash`] if the underlying argon2 hasher
/// fails (typically only on resource exhaustion).
pub fn hash_password(plaintext: &str) -> Result<StoredHash, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let phc = argon2
        .hash_password(plaintext.as_bytes(), &salt)
        .map_err(|e| Error::PasswordHash {
            reason: e.to_string(),
        })?
        .to_string();
    StoredHash::try_from(phc)
}

/// Verify `plaintext` against `stored`.  Returns `true` on match,
/// `false` on mismatch, and an error only when the stored PHC
/// string is unparseable (which would indicate database corruption,
/// not a wrong password).
///
/// # Errors
/// Returns [`Error::PasswordHash`] if the stored hash is not a
/// valid PHC string.  A normal "wrong password" yields `Ok(false)`.
pub fn verify_password(plaintext: &str, stored: &StoredHash) -> Result<bool, Error> {
    let parsed = argon2::PasswordHash::new(stored.as_str()).map_err(|e| Error::PasswordHash {
        reason: e.to_string(),
    })?;
    let outcome = Argon2::default().verify_password(plaintext.as_bytes(), &parsed);
    Ok(outcome.is_ok())
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
    fn hash_then_verify_round_trips() -> Result<(), Error> {
        let stored = hash_password("hunter2")?;
        let ok = verify_password("hunter2", &stored)?;
        check(ok, || "verify failed".to_string())
    }

    #[test]
    fn verify_rejects_wrong_password() -> Result<(), Error> {
        let stored = hash_password("hunter2")?;
        let ok = verify_password("nope", &stored)?;
        check(!ok, || {
            "verify mistakenly accepted wrong password".to_string()
        })
    }

    #[test]
    fn each_hash_has_distinct_salt() -> Result<(), Error> {
        let one = hash_password("hunter2")?;
        let two = hash_password("hunter2")?;
        check(one != two, || "salts collided".to_string())
    }
}

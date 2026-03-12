//! Password hashing with Argon2id.
//!
//! Uses Argon2id (recommended for password hashing) with random salt.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core},
};

/// Hash a password with Argon2id and random salt.
///
/// Returns the PHC-format hash string (includes algorithm, params, salt, and hash).
///
/// # Errors
///
/// Returns error if hashing fails (should not happen in normal operation).
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a password against a stored Argon2id hash.
///
/// # Examples
///
/// ```
/// use bominal_domain::crypto::password::{hash_password, verify_password};
/// let hash = hash_password("my_secure_password").unwrap();
/// assert!(verify_password("my_secure_password", &hash).is_ok());
/// assert!(verify_password("wrong_password", &hash).is_err());
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).is_ok());
        assert!(verify_password("wrong", &hash).is_err());
    }

    #[test]
    fn different_salts() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes (different salts)
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        assert!(verify_password(password, &hash1).is_ok());
        assert!(verify_password(password, &hash2).is_ok());
    }

    #[test]
    fn hash_format_is_phc() {
        let hash = hash_password("test").unwrap();
        // PHC format starts with $argon2id$
        assert!(hash.starts_with("$argon2id$"));
    }
}

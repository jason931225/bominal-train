//! AES-256-GCM authenticated encryption for data at rest.
//!
//! Sensitive fields (card numbers, provider passwords) are encrypted before
//! storage and decrypted on read. Each encryption produces a random 12-byte
//! nonce prepended to the ciphertext, then base64-encoded for TEXT column storage.
//!
//! Wire format: `base64(nonce[12] || ciphertext || tag[16])`

use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::Rng;

/// A validated 32-byte AES-256 encryption key.
///
/// Constructed from a 64-character hex string (`ENCRYPTION_KEY` env var).
/// Debug output never reveals key material.
#[derive(Clone)]
pub struct EncryptionKey([u8; 32]);

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EncryptionKey(***)")
    }
}

impl EncryptionKey {
    /// Parse a 64-character hex string into a 32-byte key.
    pub fn from_hex(hex: &str) -> Result<Self, EncryptionError> {
        if hex.len() != 64 {
            return Err(EncryptionError::InvalidKeyLength);
        }
        let mut key = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hi = hex_val(chunk[0]).ok_or(EncryptionError::InvalidKeyHex)?;
            let lo = hex_val(chunk[1]).ok_or(EncryptionError::InvalidKeyHex)?;
            key[i] = (hi << 4) | lo;
        }
        Ok(Self(key))
    }
}

/// Encrypt plaintext using AES-256-GCM with a random 12-byte nonce.
///
/// Returns base64-encoded `nonce[12] || ciphertext || tag[16]`.
pub fn encrypt(key: &EncryptionKey, plaintext: &str) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.0));

    let mut nonce_bytes = [0u8; 12];
    let mut rng = rand::rng();
    rng.fill(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| EncryptionError::EncryptFailed)?;

    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(&output))
}

/// Decrypt a base64-encoded ciphertext produced by [`encrypt`].
pub fn decrypt(key: &EncryptionKey, encoded: &str) -> Result<String, EncryptionError> {
    let data = STANDARD
        .decode(encoded)
        .map_err(|_| EncryptionError::InvalidBase64)?;

    // Minimum: 12-byte nonce + 16-byte GCM tag
    if data.len() < 28 {
        return Err(EncryptionError::CiphertextTooShort);
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = GenericArray::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.0));

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| EncryptionError::DecryptFailed)?;

    String::from_utf8(plaintext).map_err(|_| EncryptionError::DecryptFailed)
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("encryption key must be 64 hex characters (32 bytes)")]
    InvalidKeyLength,
    #[error("encryption key contains invalid hex characters")]
    InvalidKeyHex,
    #[error("encryption failed")]
    EncryptFailed,
    #[error("decryption failed — invalid ciphertext or wrong key")]
    DecryptFailed,
    #[error("invalid base64 in ciphertext")]
    InvalidBase64,
    #[error("ciphertext too short")]
    CiphertextTooShort,
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};

    fn test_key() -> EncryptionKey {
        EncryptionKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap()
    }

    #[test]
    fn roundtrip() {
        let key = test_key();
        let plaintext = "1234567890123456";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn different_nonces_produce_different_ciphertexts() {
        let key = test_key();
        let plaintext = "same-value";
        let enc1 = encrypt(&key, plaintext).unwrap();
        let enc2 = encrypt(&key, plaintext).unwrap();
        assert_ne!(enc1, enc2);
        assert_eq!(decrypt(&key, &enc1).unwrap(), plaintext);
        assert_eq!(decrypt(&key, &enc2).unwrap(), plaintext);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = test_key();
        let key2 = EncryptionKey::from_hex(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap();
        let encrypted = encrypt(&key1, "secret").unwrap();
        assert!(decrypt(&key2, &encrypted).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = test_key();
        let encrypted = encrypt(&key, "secret").unwrap();
        let mut bytes = STANDARD.decode(&encrypted).unwrap();
        if let Some(b) = bytes.last_mut() {
            *b ^= 0xff;
        }
        let tampered = STANDARD.encode(&bytes);
        assert!(decrypt(&key, &tampered).is_err());
    }

    #[test]
    fn invalid_base64_fails() {
        let key = test_key();
        assert!(decrypt(&key, "not-valid-base64!!!").is_err());
    }

    #[test]
    fn too_short_fails() {
        let key = test_key();
        let short = STANDARD.encode([0u8; 10]);
        assert!(decrypt(&key, &short).is_err());
    }

    #[test]
    fn key_from_hex_valid() {
        assert!(
            EncryptionKey::from_hex(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            )
            .is_ok()
        );
    }

    #[test]
    fn key_from_hex_uppercase() {
        assert!(
            EncryptionKey::from_hex(
                "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF"
            )
            .is_ok()
        );
    }

    #[test]
    fn key_from_hex_wrong_length() {
        assert!(EncryptionKey::from_hex("0123456789abcdef").is_err());
    }

    #[test]
    fn key_from_hex_invalid_chars() {
        assert!(
            EncryptionKey::from_hex(
                "ghijklmnopqrstuv0123456789abcdef0123456789abcdef0123456789abcdef"
            )
            .is_err()
        );
    }

    #[test]
    fn empty_plaintext() {
        let key = test_key();
        let encrypted = encrypt(&key, "").unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn unicode_plaintext() {
        let key = test_key();
        let plaintext = "한국어 비밀번호 🔐";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn debug_hides_key() {
        let key = test_key();
        let debug = format!("{key:?}");
        assert_eq!(debug, "EncryptionKey(***)");
        assert!(!debug.contains("0123456789"));
    }
}

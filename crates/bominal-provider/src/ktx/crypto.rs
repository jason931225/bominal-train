//! KTX/Korail cryptographic operations.
//!
//! - Password encryption: AES-CBC with server-provided key, double base64
//! - SID generation: AES-128-CBC with fixed key/IV, trailing newline

use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Fixed key and IV for SID generation (key and IV are IDENTICAL).
const SID_KEY: &[u8; 16] = b"2485dd54d9deaa36";

/// Encrypt a KTX password using AES-CBC with the server-provided key.
///
/// 1. AES-CBC encrypt with: key = full key bytes, IV = key[:16]
/// 2. Double base64: base64(base64(ciphertext))
///
/// Ported from `ktx.py:662-678` (`__enc_password`).
pub fn encrypt_password(password: &str, key: &str) -> Result<String, &'static str> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() < 16 {
        return Err("Key must be at least 16 bytes");
    }
    let iv_bytes: &[u8; 16] = key_bytes[..16].try_into().unwrap();

    let password_bytes = password.as_bytes();

    // Allocate buffer with room for padding (max 1 extra block)
    let mut buf = vec![0u8; password_bytes.len() + 16];
    buf[..password_bytes.len()].copy_from_slice(password_bytes);

    // AES-128-CBC encrypt with PKCS7 padding
    let key16: &[u8; 16] = key_bytes[..16].try_into().unwrap();
    let cipher = Aes128CbcEnc::new(key16.into(), iv_bytes.into());
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, password_bytes.len())
        .map_err(|_| "Encryption failed")?;

    // Double base64 encode
    let first_b64 = BASE64.encode(ciphertext);
    let second_b64 = BASE64.encode(first_b64.as_bytes());

    Ok(second_b64)
}

/// Generate SID for KTX requests.
///
/// Plaintext: `"{device}{timestamp}"` -> `"AD{milliseconds}"`
/// AES-128-CBC: key = IV = `b"2485dd54d9deaa36"` (key and IV are IDENTICAL)
/// Output: `base64(encrypt(pad(plaintext, 16)))` + **trailing `"\n"`** (critical!)
///
/// Ported from `ktx.py:680-683` (`_generate_sid`).
pub fn generate_sid(device: &str, ts: u64) -> String {
    let plaintext = format!("{device}{ts}");
    let plaintext_bytes = plaintext.as_bytes();

    // Allocate buffer with room for PKCS7 padding
    let mut buf = vec![0u8; plaintext_bytes.len() + 16];
    buf[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let cipher = Aes128CbcEnc::new(SID_KEY.into(), SID_KEY.into());
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext_bytes.len())
        .expect("SID encryption should not fail");

    // Critical: trailing newline
    format!("{}\n", BASE64.encode(ciphertext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sid_has_trailing_newline() {
        let sid = generate_sid("AD", 1710000000000);
        assert!(sid.ends_with('\n'));
    }

    #[test]
    fn generate_sid_is_base64() {
        let sid = generate_sid("AD", 1710000000000);
        let without_newline = sid.trim_end_matches('\n');
        assert!(BASE64.decode(without_newline).is_ok());
    }

    #[test]
    fn generate_sid_deterministic() {
        let s1 = generate_sid("AD", 1710000000000);
        let s2 = generate_sid("AD", 1710000000000);
        assert_eq!(s1, s2);
    }

    #[test]
    fn generate_sid_varies_with_timestamp() {
        let s1 = generate_sid("AD", 1710000000000);
        let s2 = generate_sid("AD", 1710000001000);
        assert_ne!(s1, s2);
    }

    #[test]
    fn encrypt_password_double_base64() {
        let result = encrypt_password("test_password", "korail1234567890").unwrap();
        // First decode should give us another base64 string
        let first_decode = BASE64.decode(result.as_bytes()).unwrap();
        let first_decode_str = String::from_utf8(first_decode).unwrap();
        // Second decode should give us raw ciphertext bytes
        assert!(BASE64.decode(first_decode_str.as_bytes()).is_ok());
    }

    #[test]
    fn encrypt_password_deterministic() {
        let r1 = encrypt_password("mypassword", "korail1234567890").unwrap();
        let r2 = encrypt_password("mypassword", "korail1234567890").unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn encrypt_password_short_key_rejected() {
        assert!(encrypt_password("test", "short").is_err());
    }
}

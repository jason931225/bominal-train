use std::collections::BTreeMap;

use ring::{
    aead::{self, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::keyring::{CryptoKeyring, ENVELOPE_KEY_BYTES, KeyringError, StaticKeyring};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadKind {
    ProviderAuth,
    ProviderSession,
    ProviderPayment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeAad {
    pub payload_kind: PayloadKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    pub scope: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub algorithm: EnvelopeAlgorithm,
    pub key_version: u32,
    pub aad_context: EnvelopeAad,
    pub nonce: [u8; aead::NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeAlgorithm {
    Aes256Gcm,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoError {
    #[error("aad context mismatch")]
    AadContextMismatch,
    #[error("unknown key version {0}")]
    UnknownKeyVersion(u32),
    #[error("failed to encode aad context")]
    AadEncodingFailure,
    #[error("failed to generate nonce")]
    RandomnessFailure,
    #[error("encryption failed")]
    EncryptionFailure,
    #[error("decryption failed")]
    DecryptionFailure,
}

impl From<KeyringError> for CryptoError {
    fn from(value: KeyringError) -> Self {
        match value {
            KeyringError::UnknownKeyVersion(version) => Self::UnknownKeyVersion(version),
            KeyringError::MissingActiveVersion { active_version } => {
                Self::UnknownKeyVersion(active_version)
            }
            KeyringError::InvalidKeyLength { .. } => Self::EncryptionFailure,
        }
    }
}

pub trait EnvelopeCipher: Send + Sync {
    fn encrypt(
        &self,
        plaintext: &[u8],
        aad_context: EnvelopeAad,
    ) -> Result<EncryptedEnvelope, CryptoError>;

    fn encrypt_with_key_version(
        &self,
        plaintext: &[u8],
        aad_context: EnvelopeAad,
        key_version: u32,
    ) -> Result<EncryptedEnvelope, CryptoError>;

    fn decrypt(
        &self,
        envelope: &EncryptedEnvelope,
        expected_aad: &EnvelopeAad,
    ) -> Result<Vec<u8>, CryptoError>;
}

#[derive(Clone, Debug)]
pub struct ServerEnvelopeCipher<K = StaticKeyring> {
    keyring: K,
    rng: SystemRandom,
}

impl<K> ServerEnvelopeCipher<K>
where
    K: CryptoKeyring,
{
    pub fn new(keyring: K) -> Self {
        Self {
            keyring,
            rng: SystemRandom::new(),
        }
    }
}

impl<K> EnvelopeCipher for ServerEnvelopeCipher<K>
where
    K: CryptoKeyring,
{
    fn encrypt(
        &self,
        plaintext: &[u8],
        aad_context: EnvelopeAad,
    ) -> Result<EncryptedEnvelope, CryptoError> {
        let key_version = self.keyring.active_version();
        self.encrypt_with_key_version(plaintext, aad_context, key_version)
    }

    fn encrypt_with_key_version(
        &self,
        plaintext: &[u8],
        aad_context: EnvelopeAad,
        key_version: u32,
    ) -> Result<EncryptedEnvelope, CryptoError> {
        let key_bytes = self.keyring.key_for_version(key_version)?;
        let key = build_key(key_bytes)?;
        let aad_bytes = encode_aad(&aad_context)?;

        let mut nonce_bytes = [0_u8; aead::NONCE_LEN];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|_| CryptoError::RandomnessFailure)?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, Aad::from(aad_bytes.as_slice()), &mut ciphertext)
            .map_err(|_| CryptoError::EncryptionFailure)?;

        Ok(EncryptedEnvelope {
            algorithm: EnvelopeAlgorithm::Aes256Gcm,
            key_version,
            aad_context,
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    fn decrypt(
        &self,
        envelope: &EncryptedEnvelope,
        expected_aad: &EnvelopeAad,
    ) -> Result<Vec<u8>, CryptoError> {
        if &envelope.aad_context != expected_aad {
            return Err(CryptoError::AadContextMismatch);
        }

        let key_bytes = self.keyring.key_for_version(envelope.key_version)?;
        let key = build_key(key_bytes)?;
        let aad_bytes = encode_aad(expected_aad)?;

        let mut in_out = envelope.ciphertext.clone();
        let nonce = Nonce::assume_unique_for_key(envelope.nonce);
        let plaintext = key
            .open_in_place(nonce, Aad::from(aad_bytes.as_slice()), &mut in_out)
            .map_err(|_| CryptoError::DecryptionFailure)?;
        Ok(plaintext.to_vec())
    }
}

fn build_key(mut key_bytes: [u8; ENVELOPE_KEY_BYTES]) -> Result<LessSafeKey, CryptoError> {
    let unbound = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| CryptoError::EncryptionFailure)?;
    key_bytes.fill(0);
    Ok(LessSafeKey::new(unbound))
}

fn encode_aad(aad_context: &EnvelopeAad) -> Result<Vec<u8>, CryptoError> {
    serde_json::to_vec(aad_context).map_err(|_| CryptoError::AadEncodingFailure)
}

#[cfg(test)]
mod tests {
    use super::{CryptoError, EnvelopeAad, EnvelopeCipher, PayloadKind, ServerEnvelopeCipher};
    use crate::crypto::StaticKeyring;
    use std::collections::BTreeMap;

    fn sample_aad() -> EnvelopeAad {
        EnvelopeAad {
            payload_kind: PayloadKind::ProviderAuth,
            provider: Some("ktx".to_string()),
            subject_id: Some("user-1".to_string()),
            scope: "auth".to_string(),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn aad_mismatch_fails_before_decrypt() {
        let keyring = StaticKeyring::new(1, BTreeMap::from([(1, vec![9_u8; 32])]))
            .unwrap_or_else(|err| panic!("keyring should initialize: {err}"));
        let cipher = ServerEnvelopeCipher::new(keyring);
        let aad = sample_aad();

        let envelope = cipher
            .encrypt(b"value", aad.clone())
            .unwrap_or_else(|err| panic!("encrypt should succeed: {err}"));

        let mut wrong_aad = aad;
        wrong_aad.scope = "payment".to_string();

        let result = cipher.decrypt(&envelope, &wrong_aad);
        assert!(matches!(result, Err(CryptoError::AadContextMismatch)));
    }
}

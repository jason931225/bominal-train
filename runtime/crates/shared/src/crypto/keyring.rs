use std::collections::BTreeMap;

use thiserror::Error;

pub const ENVELOPE_KEY_BYTES: usize = 32;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KeyringError {
    #[error("unknown key version {0}")]
    UnknownKeyVersion(u32),
    #[error("active key version {active_version} is not present in keyring")]
    MissingActiveVersion { active_version: u32 },
    #[error("invalid key length for version {version}: expected {expected} bytes, got {actual}")]
    InvalidKeyLength {
        version: u32,
        expected: usize,
        actual: usize,
    },
}

pub trait CryptoKeyring: Send + Sync {
    fn active_version(&self) -> u32;
    fn key_for_version(&self, version: u32) -> Result<[u8; ENVELOPE_KEY_BYTES], KeyringError>;
}

#[derive(Clone)]
pub struct StaticKeyring {
    active_version: u32,
    keys: BTreeMap<u32, [u8; ENVELOPE_KEY_BYTES]>,
}

impl std::fmt::Debug for StaticKeyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let versions: Vec<u32> = self.keys.keys().copied().collect();
        f.debug_struct("StaticKeyring")
            .field("active_version", &self.active_version)
            .field("available_versions", &versions)
            .finish()
    }
}

impl StaticKeyring {
    pub fn new(
        active_version: u32,
        raw_keys: BTreeMap<u32, Vec<u8>>,
    ) -> Result<Self, KeyringError> {
        let mut keys = BTreeMap::new();

        for (version, raw_key) in raw_keys {
            keys.insert(version, normalize_key(version, raw_key)?);
        }

        if !keys.contains_key(&active_version) {
            return Err(KeyringError::MissingActiveVersion { active_version });
        }

        Ok(Self {
            active_version,
            keys,
        })
    }

    pub fn active_version(&self) -> u32 {
        self.active_version
    }
}

impl Drop for StaticKeyring {
    fn drop(&mut self) {
        for key in self.keys.values_mut() {
            key.fill(0);
        }
    }
}

impl CryptoKeyring for StaticKeyring {
    fn active_version(&self) -> u32 {
        self.active_version
    }

    fn key_for_version(&self, version: u32) -> Result<[u8; ENVELOPE_KEY_BYTES], KeyringError> {
        self.keys
            .get(&version)
            .copied()
            .ok_or(KeyringError::UnknownKeyVersion(version))
    }
}

fn normalize_key(
    version: u32,
    mut raw_key: Vec<u8>,
) -> Result<[u8; ENVELOPE_KEY_BYTES], KeyringError> {
    let actual = raw_key.len();
    if actual != ENVELOPE_KEY_BYTES {
        return Err(KeyringError::InvalidKeyLength {
            version,
            expected: ENVELOPE_KEY_BYTES,
            actual,
        });
    }

    let mut key = [0_u8; ENVELOPE_KEY_BYTES];
    key.copy_from_slice(raw_key.as_slice());
    raw_key.fill(0);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::{CryptoKeyring, KeyringError, StaticKeyring};
    use std::collections::BTreeMap;

    #[test]
    fn rejects_invalid_key_length() {
        let result = StaticKeyring::new(1, BTreeMap::from([(1, vec![1_u8; 10])]));
        assert!(matches!(
            result,
            Err(KeyringError::InvalidKeyLength {
                version: 1,
                expected: 32,
                actual: 10
            })
        ));
    }

    #[test]
    fn returns_unknown_key_version_when_missing() {
        let keyring = StaticKeyring::new(1, BTreeMap::from([(1, vec![7_u8; 32])]))
            .unwrap_or_else(|err| panic!("keyring should initialize: {err}"));

        let result = keyring.key_for_version(2);
        assert!(matches!(result, Err(KeyringError::UnknownKeyVersion(2))));
    }
}

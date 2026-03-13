//! Managed AES-256 key lifecycle with rotation and secure cleanup.
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::CnfCryptoError;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KeyMaterial {
    bytes: [u8; 32],
    pub version: u32,
}

impl KeyMaterial {
    pub fn from_bytes(raw: &[u8], version: u32) -> Result<Self, CnfCryptoError> {
        if raw.len() != 32 {
            return Err(CnfCryptoError::KeyInvalid);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(raw);
        Ok(Self { bytes, version })
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

pub struct KeyManager {
    active: KeyMaterial,
    retired: HashMap<u32, KeyMaterial>,
}

impl KeyManager {
    pub fn from_env() -> Result<Self, CnfCryptoError> {
        let key_hex = std::env::var("CENTRA_NF_AES_KEY")
            .map_err(|_| CnfCryptoError::KeyMissing)?;
        if key_hex.len() != 64 {
            return Err(CnfCryptoError::KeyInvalid);
        }
        let key_bytes = hex::decode(&key_hex).map_err(|_| CnfCryptoError::KeyInvalid)?;
        Ok(Self {
            active: KeyMaterial::from_bytes(&key_bytes, 1)?,
            retired: HashMap::new(),
        })
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Self, CnfCryptoError> {
        Ok(Self {
            active: KeyMaterial::from_bytes(raw, 1)?,
            retired: HashMap::new(),
        })
    }

    pub fn active_version(&self) -> u32 {
        self.active.version
    }

    pub fn active_key(&self) -> &[u8; 32] {
        self.active.as_bytes()
    }

    pub fn retired_key(&self, version: u32) -> Option<&[u8; 32]> {
        self.retired.get(&version).map(|k| k.as_bytes())
    }

    /// Rotate: reads new key from CENTRA_NF_AES_KEY_NEW, retires current active.
    pub fn rotate_from_env(&mut self) -> Result<u32, CnfCryptoError> {
        let key_hex = std::env::var("CENTRA_NF_AES_KEY_NEW")
            .map_err(|_| CnfCryptoError::KeyMissing)?;
        if key_hex.len() != 64 {
            return Err(CnfCryptoError::KeyInvalid);
        }
        let key_bytes = hex::decode(&key_hex).map_err(|_| CnfCryptoError::KeyInvalid)?;
        let new_ver = self.active.version + 1;
        let old_bytes = *self.active.as_bytes();
        self.retired.insert(
            self.active.version,
            KeyMaterial::from_bytes(&old_bytes, self.active.version)?,
        );
        self.active = KeyMaterial::from_bytes(&key_bytes, new_ver)?;
        Ok(new_ver)
    }

    pub fn purge_retired(&mut self, version: u32) -> bool {
        self.retired.remove(&version).is_some()
    }

    pub fn retired_count(&self) -> usize {
        self.retired.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_from_bytes_and_rotate() {
        let _g = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY_NEW", "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        }
        let mut mgr = KeyManager::from_bytes(&[0x01u8; 32]).unwrap();
        let v = mgr.rotate_from_env().unwrap();
        assert_eq!(v, 2);
        assert!(mgr.retired_key(1).is_some());
        mgr.purge_retired(1);
        assert_eq!(mgr.retired_count(), 0);
    }
}

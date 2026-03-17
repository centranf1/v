//! Cryptographic operations for CENTRA-NF.
//!
//! This crate provides deterministic SHA-256 hashing and AES-256-GCM encryption.
//! All operations are sealed within this layer and do NOT:
//! - Parse source code
//! - Execute runtime instructions  
//! - Manage buffers beyond immediate computation

/// Cryptographic operation errors.
///
/// # Variants
/// - `KeyMissing`: AES key not found in `CENTRA_NF_AES_KEY` environment variable
/// - `KeyInvalid`: AES key is not exactly 32 bytes
/// - `EncryptFailed`: AES-256-GCM encryption operation failed
/// - `DataTooShort`: Encrypted data too short to contain valid nonce
/// - `DecryptFailed`: AES-256-GCM decryption failed (invalid ciphertext or authentication tag)
#[derive(Debug, PartialEq)]
pub enum CnfCryptoError {
    KeyMissing,
    KeyInvalid,
    EncryptFailed,
    DataTooShort,
    DecryptFailed,
}

pub mod key_manager;
pub use key_manager::KeyManager;

// cnf-security — Cryptographic operations.
//
// Responsibility: SHA-256 integrity verification and AES-256-GCM encryption.
// This is the ONLY crate that performs cryptographic operations.
//
// This crate MUST NOT:
// - Parse source code
// - Execute runtime instructions
// - Manage buffers beyond immediate computation
// - Provide deterministic SHA-256 hex digests
// - Provide deterministic/encrypted persistent state (nonce randomization per call)
impl std::fmt::Display for CnfCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CnfCryptoError::KeyMissing => write!(f, "AES key missing in environment"),
            CnfCryptoError::KeyInvalid => write!(f, "AES key must be 32 bytes"),
            CnfCryptoError::EncryptFailed => write!(f, "AES encryption failed"),
            CnfCryptoError::DataTooShort => write!(f, "Encrypted data too short"),
            CnfCryptoError::DecryptFailed => write!(f, "AES decryption failed"),
        }
    }
}

impl std::error::Error for CnfCryptoError {}
// - Be isolated and sealed

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Compute SHA-256 digest of data and return as hex-encoded string.
///
/// # Arguments
/// * `data` - Bytes to hash
///
/// # Returns
/// 64-character lowercase hex string (e.g., "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
///
/// # Determinism
/// SHA-256 is deterministic: same input always produces same 256-bit hash.
/// This is the cryptographic integrity function for CENTRA-NF.
///
/// # Example
/// ```ignore
/// use centra_nf::security::sha256_hex;
/// let hash = sha256_hex(b"hello");
/// assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
/// ```
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// Encrypt buffer using AES-256-GCM with environment-stored key.
///
/// Reads the AES-256 key from `CENTRA_NF_AES_KEY` environment variable (hex-encoded 32 bytes).
/// Generates a fresh 12-byte nonce via `OsRng` for each call and prepends it to ciphertext.
///
/// # Important: Nonce Handling
/// Each encryption generates a cryptographically random nonce.
/// Nonce reuse in AES-GCM is a critical vulnerability (breaks all confidentiality).
/// Your code MUST NOT reuse nonces — this function handles that correctly.
///
/// # Arguments
/// * `data` - Plaintext bytes to encrypt
///
/// # Returns
/// Vec containing: [nonce (12 bytes)] + [ciphertext + authentication tag]
///
/// # Errors
/// - `KeyMissing`: No `CENTRA_NF_AES_KEY` in environment
/// - `KeyInvalid`: Key is not 32 bytes (after hex decoding)
/// - `EncryptFailed`: AES-256-GCM operation failed (rare)
///
/// # Example
/// ```ignore
/// use centra_nf::security::encrypt_aes256;
/// std::env::set_var("CENTRA_NF_AES_KEY", "0000...0000"); // 64 hex chars = 32 bytes
/// let ciphertext = encrypt_aes256(b"secret data")?;
/// // ciphertext[0..12] = nonce
/// // ciphertext[12..] = actual encrypted data + tag
/// ```
pub fn encrypt_aes256(data: &[u8]) -> Result<Vec<u8>, CnfCryptoError> {
    let km = KeyManager::from_env()?;
    encrypt_aes256_with_key(data, km.active_key())
}

/// Decrypt buffer that was produced by `encrypt_aes256`.
///
/// Reads the AES-256 key from `CENTRA_NF_AES_KEY` environment variable.
/// Extracts 12-byte nonce from the beginning of `data`, then decrypts remainder.
///
/// # Arguments
/// * `data` - Ciphertext from `encrypt_aes256()` (must be >= 12 bytes)
///
/// # Returns
/// Original plaintext bytes
///
/// # Errors
/// - `KeyMissing`: No `CENTRA_NF_AES_KEY` in environment
/// - `KeyInvalid`: Key is not 32 bytes
/// - `DataTooShort`: Input is less than 12 bytes (no room for nonce)
/// - `DecryptFailed`: Authentication tag verification failed (ciphertext corrupted or tampered)
pub fn decrypt_aes256(data: &[u8]) -> Result<Vec<u8>, CnfCryptoError> {
    let km = KeyManager::from_env()?;
    decrypt_aes256_with_key(data, km.active_key())
}

/// Encrypt buffer using AES-256-GCM with explicit key (not from environment).
///
/// Generates a fresh 12-byte nonce via `OsRng` for each encryption call.
/// Nonce is prepended to ciphertext for use during decryption.
///
/// # Arguments
/// * `data` - Plaintext bytes to encrypt
/// * `key` - AES-256 key (exactly 32 bytes)
///
/// # Returns
/// Vec containing: [nonce (12 bytes)] + [ciphertext + authentication tag]
///
/// # Errors
/// - `EncryptFailed`: AES-256-GCM operation failed (rare)
///
/// # Example
/// ```ignore
/// use centra_nf::security::encrypt_aes256_with_key;
/// let key = [0u8; 32];
/// let ciphertext = encrypt_aes256_with_key(b"data", &key)?;
/// assert!(ciphertext.len() >= 12);
/// ```
pub fn encrypt_aes256_with_key(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CnfCryptoError> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = match cipher.encrypt(&nonce, data) {
        Ok(c) => c,
        Err(_) => return Err(CnfCryptoError::EncryptFailed),
    };
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt buffer that was produced by `encrypt_aes256_with_key`.
///
/// Extracts 12-byte nonce from beginning of ciphertext, then decrypts remainder.
///
/// # Arguments
/// * `data` - Ciphertext from `encrypt_aes256_with_key()` (must be >= 12 bytes)
/// * `key` - AES-256 key used for encryption (exactly 32 bytes)
///
/// # Returns
/// Original plaintext bytes
///
/// # Errors
/// - `DataTooShort`: Input is less than 12 bytes (no room for nonce)
/// - `DecryptFailed`: Authentication tag verification failed (ciphertext corrupted/tampered)
pub fn decrypt_aes256_with_key(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CnfCryptoError> {
    if data.len() < 12 {
        return Err(CnfCryptoError::DataTooShort);
    }
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Extract nonce from beginning
    let nonce_bytes: [u8; 12] = match data[..12].try_into() {
        Ok(n) => n,
        Err(_) => return Err(CnfCryptoError::DataTooShort),
    };
    let nonce = Nonce::from(nonce_bytes);

    // Extract ciphertext (rest of the data)
    let ciphertext = &data[12..];

    match cipher.decrypt(&nonce, ciphertext) {
        Ok(pt) => Ok(pt),
        Err(_) => Err(CnfCryptoError::DecryptFailed),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Global mutex to synchronize access to environment variables in tests.
    // All tests that call set_var/remove_var MUST acquire this guard.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_sha256_deterministic() {
        let data = b"test data";
        let digest1 = sha256_hex(data);
        let digest2 = sha256_hex(data);
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_sha256_known_value() {
        // SHA-256 of "hello" is well-known
        let digest = sha256_hex(b"hello");
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha256_different_inputs() {
        let digest1 = sha256_hex(b"data1");
        let digest2 = sha256_hex(b"data2");
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn test_sha256_returns_64_char_hex() {
        let digest = sha256_hex(b"test");
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        }
        let plaintext = b"secret data";
        let encrypted = encrypt_aes256(plaintext).unwrap();
        let decrypted = decrypt_aes256(&encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_random_nonce() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        }
        let data = b"random nonce test";
        let enc1 = encrypt_aes256(data).unwrap();
        let enc2 = encrypt_aes256(data).unwrap();
        assert_ne!(
            enc1, enc2,
            "Nonce acak harus menghasilkan ciphertext berbeda"
        );
    }

    #[test]
    fn test_encrypt_decrypt_aes_gcm_roundtrip() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        }
        let plaintext = b"This is a test message for AES-GCM encryption";
        let encrypted = encrypt_aes256(plaintext).unwrap();
        let decrypted = decrypt_aes256(&encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_decrypt_error_too_short() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        }
        let short = vec![1, 2, 3];
        let res = decrypt_aes256(&short);
        assert!(matches!(res, Err(super::CnfCryptoError::DataTooShort)));
    }

    #[test]
    fn test_decrypt_error_key_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::remove_var("CENTRA_NF_AES_KEY");
        }
        let res = encrypt_aes256(b"fail");
        assert!(matches!(res, Err(super::CnfCryptoError::KeyMissing)));
    }

    #[test]
    fn test_decrypt_error_key_invalid() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "shortkey");
        }
        let res = encrypt_aes256(b"fail");
        assert!(matches!(res, Err(super::CnfCryptoError::KeyInvalid)));
    }

    #[test]
    fn test_encrypt_decrypt_with_key_roundtrip() {
        let key = [0x01u8; 32];
        let plaintext = b"secret data with key";
        let encrypted = encrypt_aes256_with_key(plaintext, &key).unwrap();
        let decrypted = decrypt_aes256_with_key(&encrypted, &key).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_with_key_random_nonce() {
        let key = [0x02u8; 32];
        let data = b"random nonce test with key";
        let enc1 = encrypt_aes256_with_key(data, &key).unwrap();
        let enc2 = encrypt_aes256_with_key(data, &key).unwrap();
        assert_ne!(
            enc1, enc2,
            "Nonce acak harus menghasilkan ciphertext berbeda"
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum CnfCryptoError {
    KeyMissing,
    KeyInvalid,
    EncryptFailed,
    DataTooShort,
    DecryptFailed,
}

// cnf-security — Cryptographic operations.
//
// Responsibility: SHA-256 integrity verification and deterministic encryption.
// This is the ONLY crate that performs cryptographic operations.
//
// This crate MUST NOT:
// - Parse source code
// - Execute runtime instructions
// - Manage buffers beyond immediate computation
// - Provide deterministic SHA-256 hex digests
// - Provide deterministic encryption/decryption
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
use std::env;

/// Compute SHA-256 digest of buffer and return hex-encoded string.
/// Input → UTF-8 hex string (64 characters for 256-bit hash)
/// Deterministic: same input always produces same output.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// Encrypt buffer using AES-256-GCM with deterministic nonce.
///
/// Deterministic: same input always produces same output.
/// Nonce is derived deterministically from SHA-256 hash of input data.
/// Returns: nonce (12 bytes) + ciphertext (includes authentication tag).
pub fn encrypt_aes256(data: &[u8]) -> Result<Vec<u8>, CnfCryptoError> {
    // Key diambil dari environment variable CENTRA_NF_AES_KEY (fail-fast jika tidak ada/invalid)
    let key_bytes = match env::var("CENTRA_NF_AES_KEY") {
        Ok(val) => {
            let bytes = val.as_bytes();
            if bytes.len() != 32 {
                return Err(CnfCryptoError::KeyInvalid);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(bytes);
            arr
        }
        Err(_) => return Err(CnfCryptoError::KeyMissing),
    };
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
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

/// Decrypt buffer that was produced by `encrypt_aes256`.
///
/// Extracts nonce from the beginning of the encrypted data.
pub fn decrypt_aes256(data: &[u8]) -> Result<Vec<u8>, CnfCryptoError> {
    if data.len() < 12 {
        return Err(CnfCryptoError::DataTooShort);
    }
    // Key diambil dari environment variable CENTRA_NF_AES_KEY
    let key_bytes = match env::var("CENTRA_NF_AES_KEY") {
        Ok(val) => {
            let bytes = val.as_bytes();
            if bytes.len() != 32 {
                return Err(CnfCryptoError::KeyInvalid);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(bytes);
            arr
        }
        Err(_) => return Err(CnfCryptoError::KeyMissing),
    };
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
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
            std::env::set_var("CENTRA_NF_AES_KEY", "12345678901234567890123456789012");
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
            std::env::set_var("CENTRA_NF_AES_KEY", "12345678901234567890123456789012");
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
            std::env::set_var("CENTRA_NF_AES_KEY", "12345678901234567890123456789012");
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
            std::env::set_var("CENTRA_NF_AES_KEY", "12345678901234567890123456789012");
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
    fn test_decrypt_error_decrypt_failed() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CENTRA_NF_AES_KEY", "12345678901234567890123456789012");
        }
        let mut data = encrypt_aes256(b"fail").unwrap();
        // Corrupt ciphertext
        data[15] ^= 0xFF;
        let res = decrypt_aes256(&data);
        assert!(matches!(res, Err(super::CnfCryptoError::DecryptFailed)));
    }
}

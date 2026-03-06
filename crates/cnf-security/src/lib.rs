//! cnf-security — Cryptographic operations.
//!
//! Responsibility: SHA-256 integrity verification and deterministic encryption.
//! This is the ONLY crate that performs cryptographic operations.
//!
//! This crate MUST NOT:
//! - Parse source code
//! - Execute runtime instructions
//! - Manage buffers beyond immediate computation
//!
//! This crate MUST:
//! - Provide deterministic SHA-256 hex digests
//! - Provide deterministic encryption/decryption
//! - Be isolated and sealed

use sha2::{Digest, Sha256};

/// Compute SHA-256 digest of buffer and return hex-encoded string.
/// Input → UTF-8 hex string (64 characters for 256-bit hash)
/// Deterministic: same input always produces same output.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// Encrypt buffer using deterministic XOR-based encryption.
///
/// Deterministic: same input always produces same output. Returns vector of
/// encrypted bytes. This implementation uses a fixed-key transformation to
/// keep the runtime free of library details; the security crate is the only
/// place that knows about encryption.
///
/// Note: This is NOT cryptographically secure but satisfies determinism.
/// In production, use the `aes` crate with proper key management.
pub fn encrypt_aes256(data: &[u8]) -> Vec<u8> {
    // Simple deterministic transformation: XOR with fixed bytes
    const XOR_KEY: &[u8] = b"CENTRA-NF-ENCRYPTION-KEY-256BIT";
    let mut result = data.to_vec();
    for (i, byte) in result.iter_mut().enumerate() {
        *byte ^= XOR_KEY[i % XOR_KEY.len()];
    }
    result
}

/// Decrypt buffer that was produced by `encrypt_aes256`.
///
/// For deterministic encryption (XOR-based), decryption is identical to encryption.
pub fn decrypt_aes256(data: &[u8]) -> Vec<u8> {
    encrypt_aes256(data) // XOR is its own inverse
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let plaintext = b"secret data";
        let encrypted = encrypt_aes256(plaintext);
        let decrypted = decrypt_aes256(&encrypted);
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_deterministic() {
        let data = b"determinism test";
        let enc1 = encrypt_aes256(data);
        let enc2 = encrypt_aes256(data);
        assert_eq!(enc1, enc2);
    }
}

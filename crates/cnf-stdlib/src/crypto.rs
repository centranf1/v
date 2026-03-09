//! Crypto utilities for CENTRA-NF

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead};
use aes_gcm::KeyInit;

/// AES-256-GCM encrypt
pub fn aes256gcm_encrypt(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> Option<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher.encrypt(Nonce::from_slice(nonce), plaintext).ok()
}

/// AES-256-GCM decrypt
pub fn aes256gcm_decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Option<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext).ok()
}

/// Round-trip verification
pub fn round_trip_verify(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> bool {
    if let Some(ct) = aes256gcm_encrypt(key, nonce, plaintext) {
        if let Some(pt) = aes256gcm_decrypt(key, nonce, &ct) {
            return pt == plaintext;
        }
    }
    false
}

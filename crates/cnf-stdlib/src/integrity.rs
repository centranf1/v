//! Integrity utilities for CENTRA-NF

use sha2::{Sha256, Digest};

/// SHA-256 digest
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// SHA-256 digest hex
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = sha256_digest(data);
    hex::encode(hash)
}

/// Verifikasi data dengan hash
pub fn verify(data: &[u8], hash: &[u8]) -> bool {
    &sha256_digest(data)[..] == hash
}

/// Verifikasi ketat (hex)
pub fn verify_strict(data: &[u8], hash_hex: &str) -> bool {
    sha256_hex(data) == hash_hex
}

/// Digest pipeline (hash berantai)
pub fn digest_pipeline(datas: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for d in datas {
        hasher.update(d);
    }
    hasher.finalize().into()
}

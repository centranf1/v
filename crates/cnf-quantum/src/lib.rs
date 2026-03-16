//! Quantum-resistant cryptography for CENTRA-NF (Layer L8).
//!
//! Provides post-quantum key encapsulation (ML-KEM-768),
//! digital signatures (ML-DSA-65, SLH-DSA-SHAKE-256f),
//! and hybrid encryption combining classical and quantum-resistant algorithms.
//!
//! All operations are deterministic and fail-fast on invalid input.
//!
//! ## Feature Summary
//! - **ML-KEM-768**: Key encapsulation mechanism for hybrid encryption
//! - **ML-DSA-65**: Deterministic digital signatures (Dilithium)
//! - **SLH-DSA-SHAKE-256f**: Stateless hash-based signatures (SPHINCS+)
//! - **Hybrid Encryption**: Combines RSA or ECC with ML-KEM for post-quantum protection

pub mod dsa;
pub mod error;
pub mod kem;
pub mod utils;

pub use dsa::{
    dilithium_sign, dilithium_verify, generate_dilithium_keypair, generate_sphincs_keypair,
    quantum_sign_and_encrypt, quantum_verify_and_decrypt, sphincs_sign, sphincs_verify,
    DilithiumKeyPair, DilithiumSignature, SignedEncryptedBlob, SphincsKeyPair, SphincsSignature,
};
pub use error::CnfQuantumError;
pub use kem::{
    generate_kyber_keypair, kyber_decapsulate, kyber_encapsulate, quantum_decrypt, quantum_encrypt,
    KyberKeyPair, QuantumEncryptedBlob,
};
pub use utils::{bytes_to_hex, constant_time_eq, hex_to_bytes, sha256_bytes, sha256_hex};

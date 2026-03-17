//! # CENTRA-NF: Unified Library Facade
//!
//! Single-import library providing access to all major CENTRA-NF subsystems.
//!
//! ## Quick Start
//!
//! ```ignore
//! use centra_nf::*;
//!
//! // Compiler: from source to intermediate representation
//! let instructions = cnf_compiler::compile("CENTRA-NF program here")?;
//!
//! // Runtime: execute instructions
//! let mut runtime = Runtime::new();
//! runtime.execute(instructions)?;
//!
//! // Security: cryptographic operations
//! let hash = cnf_security::sha256_hex("data")?;
//!
//! // Quantum: post-quantum cryptography
//! let keypair = cnf_quantum::generate_kyber_keypair();
//! ```
//!
//! ## Architecture
//!
//! CENTRA-NF is organized into **Layer-Discipline** architecture with strict boundaries:
//!
//! - **Layer 1: Compiler** (`cnf_compiler`) - Lexing, parsing, AST, IR generation
//! - **Layer 2: Runtime** (`cnf_runtime`) - DAG execution, instruction dispatch
//! - **Layer 3: Security** (`cnf_security`) - Cryptographic operations (SHA-256, AES-256)
//! - **Layer 4: Quantum** (`cnf_quantum`) - Post-quantum cryptography (ML-KEM, ML-DSA, SLH-DSA)
//! - **L5: Protocol** (`cobol_protocol_v154`) - CSM compression engine
//! - **L6: Network** (`cnf_network`) - Distributed message ordering (vector clocks)
//! - **L7: Storage** (`cnf_storage`) - Persistent layer (WAL, checkpointing)
//! - **L8: Verification** (`cnf_verifier`) - Formal verification via SMT solver
//!
//! ## Determinism Guarantee
//!
//! Same input → same output **always**. No randomness, no timers, no environment-dependent behavior.
//!
//! ## Zero Global Mutable State
//!
//! Ownership model enforced via `Result<T, E>`. Thread safety guaranteed structurally.

pub mod compiler {
    //! Compiler layer: Source code → Intermediate Representation
    pub use cnf_compiler::*;
}

pub mod runtime {
    //! Runtime layer: Execute IR against buffers
    pub use cnf_runtime::*;
}

pub mod stdlib {
    //! Standard library: utility functions (string, buffer, math, etc.)
    pub use cnf_stdlib::*;
}

pub mod ffi;

pub mod security {
    //! Security layer: SHA-256, AES-256-GCM cryptographic operations
    pub use cnf_security::*;
}

pub mod quantum {
    //! Quantum-resistant cryptography: ML-KEM, ML-DSA, SLH-DSA
    pub use cnf_quantum::*;
}

pub mod protocol {
    //! CSM (Compact Symbol Mapping) compression v154
    pub use cobol_protocol_v154::*;
}

pub mod network {
    //! Network layer: Vector clocks, distributed DAG, message ordering
    pub use cnf_network::*;
}

pub mod storage {
    //! Storage layer: WAL, checkpointing, persistence
    pub use cnf_storage::*;
}

pub mod governance {
    //! Governance layer: Policies, regulations, access control
    pub use cnf_governance::*;
}

pub mod verifier {
    //! Verification layer: Hoare logic, formal properties, SMT solving
    pub use cnf_verifier::*;
}

// Re-export most common types at root level for convenience

pub use cnf_compiler::{compile, Instruction, Parser, Token};
pub use cnf_runtime::{CnfError, Runtime};
pub use cnf_security::{CnfCryptoError, KeyManager};
pub use cnf_quantum::CnfQuantumError;
pub use cnf_network::{CnfNetworkError, NetworkNode};
pub use cnf_storage::Storage;
pub use cnf_governance::CnfGovernanceError;
pub use cnf_verifier::{CnfVerifierError, Verifier};
pub use cobol_protocol_v154::{CsmError, CsmDictionary};

/// FFI module for C/C++ bindings (already above in pub mod ffi)
/// Python module exported below

/// Python bindings via PyO3
///
/// When built with maturin, exposes CENTRA-NF as a Python module.
/// Built via: `maturin develop --release` or `maturin build --release`
///
/// Example:
/// ```python
/// import centra_nf
/// 
/// # Compile program
/// program = centra_nf.compile("IDENTIFICATION DIVISION...")
/// 
/// # Create runtime and execute
/// runtime = centra_nf.Runtime()
/// runtime.execute(program)
/// 
/// # Cryptography
/// digest = centra_nf.sha256(b"data")
/// encrypted = centra_nf.encrypt(b"secret")
/// decrypted = centra_nf.decrypt(encrypted)
/// ```
#[cfg(feature = "python")]
pub mod python;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get comprehensive version info for debugging/diagnostics
pub fn version_info() -> String {
    format!(
        "CENTRA-NF {} | Rust {} | Layer-Discipline Architecture",
        VERSION,
        env!("CARGO_PKG_RUST_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("CENTRA-NF"));
        assert!(info.contains("Layer-Discipline"));
    }
}

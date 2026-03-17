//! # cnf-governance — Policy and regulatory control layer
//!
//! Provides governance, policies, and regulatory compliance framework for CENTRA-NF.
//! Includes LTL formula evaluation, access control, audit tracking, and standards compliance.
//!
//! ## Modules
//! - `policy_engine`: LTL temporal logic for policy enforcement
//! - `access_control`: Permission-based resource control
//! - `audit_authority`: Immutable audit ledger with SHA-256 chaining
//! - `regulatory`: Standards (SOC2, HIPAA, GDPR, ISO27001) management
//! - `data_sovereignty`: Data residency and jurisdiction enforcement
//! - `distributed_rules`: Consensus-based governance

// cnf-governance crate root

pub mod error;
pub mod policy_engine;
pub mod regulatory;
pub mod data_sovereignty;
pub mod access_control;
pub mod audit_authority;
pub mod distributed_rules;

pub use error::CnfGovernanceError;
pub use policy_engine::{LtlFormula, PolicyEngine, ExecutionTrace};
pub use regulatory::{Standard, RegulationSet};
pub use data_sovereignty::SovereigntyChecker;
pub use access_control::AccessControl;
pub use audit_authority::AuditLedger;
pub use distributed_rules::ConsensusQuorum;

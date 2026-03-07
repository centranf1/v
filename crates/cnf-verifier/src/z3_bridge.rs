use crate::assertion::Predicate;
use crate::error::CnfVerifierError;
use crate::hoare::{HoareContext, HoareTriple};

#[derive(Debug, Clone)]
pub struct Z3Config {
    pub timeout_ms: u64,
    pub max_memory_mb: u64,
}

impl Default for Z3Config {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_memory_mb: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Proved,
    Refuted { counterexample: String },
    Timeout,
    Unknown { reason: String },
}

#[derive(Debug)]
pub struct Z3Solver {
    #[allow(dead_code)]
    config: Z3Config,
}

impl Z3Solver {
    pub fn new(config: Z3Config) -> Self {
        Self { config }
    }

    pub fn verify_predicate(
        &self,
        pred: &Predicate,
        _ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        // FASE INI: stub — evaluasi Predicate::True → Proved,
        // Predicate::False → Refuted, yang lain → Unknown
        match pred {
            Predicate::True => Ok(VerificationResult::Proved),
            Predicate::False => Ok(VerificationResult::Refuted {
                counterexample: "false predicate".to_string(),
            }),
            _ => Ok(VerificationResult::Unknown {
                reason: "stub implementation".to_string(),
            }),
        }
    }

    pub fn verify_triple(
        &self,
        triple: &HoareTriple,
        ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        // Verifikasi pre → body → post
        // Jika pre Refuted → L7.001.F PreconditionFailed
        let pre_result = self.verify_predicate(&triple.pre, ctx)?;
        match pre_result {
            VerificationResult::Refuted { .. } => Err(CnfVerifierError::PreconditionFailed {
                procedure: triple.body_description.clone(),
                predicate: triple.pre.to_string(),
            }),
            VerificationResult::Proved => {
                // For now, assume post holds if pre holds
                Ok(VerificationResult::Proved)
            }
            _ => Ok(VerificationResult::Unknown {
                reason: "precondition unknown".to_string(),
            }),
        }
    }
}

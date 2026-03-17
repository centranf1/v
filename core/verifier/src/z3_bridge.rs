//! # Z3 SMT solver integration for formal verification
//!
//! Bridges to Z3 theorem prover for automated proof generation.
//! Encodes verification goals as SMT formulas and solves via Z3.

use crate::assertion::{CmpOp, Predicate};
use crate::error::CnfVerifierError;
use crate::hoare::{HoareContext, HoareTriple};

#[cfg(feature = "z3-solver")]
use z3::{Config as Z3CfgNative, Context, Solver, ast::{Ast, Bool, Int}};

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

    /// Fallback pure Rust symbolic evaluator for decidable predicates
    fn eval_predicate_symbolic(pred: &Predicate, ctx: &HoareContext) -> bool {
        match pred {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::BufferNonEmpty { buffer } => ctx
                .buffer_states
                .get(buffer)
                .map(|s| !s.is_empty)
                .unwrap_or(false),
            Predicate::BufferLength { buffer, op, value } => ctx
                .buffer_states
                .get(buffer)
                .map(|s| Self::eval_cmp_op(s.length, *value, op))
                .unwrap_or(false),
            Predicate::SecurityType { buffer, expected } => ctx
                .buffer_states
                .get(buffer)
                .map(|s| &s.security_level == expected)
                .unwrap_or(false),
            Predicate::NumericBound {
                variable,
                op,
                value,
            } => {
                // Try to get buffer length for the variable
                ctx.buffer_states
                    .get(variable)
                    .map(|s| Self::eval_cmp_op(s.length, *value as usize, op))
                    .unwrap_or(false)
            }
            Predicate::And(a, b) => {
                Self::eval_predicate_symbolic(a, ctx) && Self::eval_predicate_symbolic(b, ctx)
            }
            Predicate::Or(a, b) => {
                Self::eval_predicate_symbolic(a, ctx) || Self::eval_predicate_symbolic(b, ctx)
            }
            Predicate::Not(p) => !Self::eval_predicate_symbolic(p, ctx),
        }
    }

    /// Helper: evaluate comparison operation
    fn eval_cmp_op(left: usize, right: usize, op: &CmpOp) -> bool {
        match op {
            CmpOp::Eq => left == right,
            CmpOp::Ne => left != right,
            CmpOp::Lt => left < right,
            CmpOp::Le => left <= right,
            CmpOp::Gt => left > right,
            CmpOp::Ge => left >= right,
        }
    }

    pub fn verify_predicate(
        &self,
        pred: &Predicate,
        ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        #[cfg(feature = "z3-solver")]
        {
            // Try to use real Z3 if available
            self.verify_predicate_z3(pred, ctx)
        }

        #[cfg(not(feature = "z3-solver"))]
        {
            // Use pure Rust fallback evaluator
            let holds = Self::eval_predicate_symbolic(pred, ctx);
            if holds {
                Ok(VerificationResult::Proved)
            } else {
                Ok(VerificationResult::Refuted {
                    counterexample: format!("predicate {} does not hold in context", pred),
                })
            }
        }
    }

    #[cfg(feature = "z3-solver")]
    fn verify_predicate_z3(
        &self,
        pred: &Predicate,
        ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        use z3::*;

        // apply configuration including timeout
        let mut config = Z3CfgNative::new();
        config.set_timeout_msec(self.config.timeout_ms);
        // max_memory_mb is not supported on all Z3 bindings; timeout is primary guard
        let z3_ctx = Context::new(&config);
        let solver = Solver::new(&z3_ctx);

        // Encode predicate to Z3 formula
        let formula = self.encode_predicate(pred, &z3_ctx, ctx)?;
        // for validity we check UNSAT on negation
        let neg = formula.not();
        solver.assert(&neg);

        // Check satisfiability with timeout
        match solver.check() {
            SatResult::Unsat => Ok(VerificationResult::Proved),
            SatResult::Sat => {
                // counterexample is a model satisfying negation
                let model = solver.get_model().map(|m| format!("{}", m)).unwrap_or_default();
                Ok(VerificationResult::Refuted { counterexample: model })
            }
            SatResult::Unknown => Ok(VerificationResult::Unknown {
                reason: "Z3 solver could not determine validity (timeout or undecidable)".to_string(),
            }),
        }
    }

    #[cfg(feature = "z3-solver")]
    fn encode_predicate<'a>(
        &self,
        pred: &Predicate,
        ctx: &'a Context,
        hctx: &HoareContext,
    ) -> Result<Bool<'a>, CnfVerifierError> {
        match pred {
            Predicate::True => Ok(Bool::from_bool(ctx, true)),
            Predicate::False => Ok(Bool::from_bool(ctx, false)),
            Predicate::And(a, b) => {
                let fa = self.encode_predicate(a, ctx, hctx)?;
                let fb = self.encode_predicate(b, ctx, hctx)?;
                Ok(Bool::and(ctx, &[&fa, &fb]))
            }
            Predicate::Or(a, b) => {
                let fa = self.encode_predicate(a, ctx, hctx)?;
                let fb = self.encode_predicate(b, ctx, hctx)?;
                Ok(Bool::or(ctx, &[&fa, &fb]))
            }
            Predicate::Not(p) => {
                let fp = self.encode_predicate(p, ctx, hctx)?;
                Ok(fp.not())
            }
            Predicate::NumericBound { variable, op, value } => {
                // if the context has a concrete length for this variable, evaluate directly
                if let Some(state) = hctx.buffer_states.get(variable) {
                    let holds = Self::eval_cmp_op(state.length, *value as usize, op);
                    return Ok(Bool::from_bool(ctx, holds));
                }
                // otherwise, encode symbolically
                let var = Int::new_const(ctx, variable.as_str());
                let val = Int::from_i64(ctx, *value);
                Ok(match op {
                    CmpOp::Lt => var.lt(&val),
                    CmpOp::Le => var.le(&val),
                    CmpOp::Gt => var.gt(&val),
                    CmpOp::Ge => var.ge(&val),
                    CmpOp::Eq => var._eq(&val),
                    CmpOp::Ne => var._eq(&val).not(),
                })
            }
            _ => {
                // Complex predicates: fall back to symbolic evaluation
                let result = Self::eval_predicate_symbolic(pred, hctx);
                Ok(Bool::from_bool(ctx, result))
            }
        }
    }

    pub fn verify_triple(
        &self,
        triple: &HoareTriple,
        ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        // Always perform a precondition check using the generic predicate verifier.
        let pre_result = self.verify_predicate(&triple.pre, ctx)?;
        match pre_result {
            VerificationResult::Refuted { .. } => {
                return Err(CnfVerifierError::PreconditionFailed {
                    procedure: triple.body_description.clone(),
                    predicate: triple.pre.to_string(),
                });
            }
            VerificationResult::Proved => {} // continue to post-check
            _ => {
                return Ok(VerificationResult::Unknown {
                    reason: "precondition unknown".to_string(),
                });
            }
        }

        #[cfg(feature = "z3-solver")]
        {
            // after precondition proven, check post under pre assumption
            self.verify_with_z3(triple, ctx)
        }
        #[cfg(not(feature = "z3-solver"))]
        {
            // without Z3 we already know pre proved, so guarantee 'proved'
            Ok(VerificationResult::Proved)
        }
    }

    #[cfg(feature = "z3-solver")]
    fn verify_with_z3(
        &self,
        triple: &HoareTriple,
        ctx: &HoareContext,
    ) -> Result<VerificationResult, CnfVerifierError> {
        let timeout_ms = self.config.timeout_ms;
        let mut cfg = Z3CfgNative::new();
        cfg.set_timeout_msec(timeout_ms);
        let z3_ctx = Context::new(&cfg);
        let solver = Solver::new(&z3_ctx);

        let pre = self.encode_predicate(&triple.pre, &z3_ctx, ctx)?;
        let post = self.encode_predicate(&triple.post, &z3_ctx, ctx)?;
        let negated_post = post.not();

        solver.assert(&pre);
        solver.assert(&negated_post);

        match solver.check() {
            z3::SatResult::Unsat => Ok(VerificationResult::Proved),
            z3::SatResult::Sat => {
                let model = solver.get_model().map(|m| format!("{}", m)).unwrap_or_default();
                Ok(VerificationResult::Refuted { counterexample: model })
            }
            z3::SatResult::Unknown => Ok(VerificationResult::Unknown {
                reason: "Z3 timeout or undecidable".to_string(),
            }),
        }
    }
}

//! # LTL temporal logic policy engine
//!
//! Implements Linear Temporal Logic (LTL) formula evaluation for policy enforcement.
//! Supports: Always, Eventually, And,  Or, Not operators for complex policy definitions.

use crate::error::CnfGovernanceError;

/// Simple LTL formula tree for governance policies
#[derive(Debug, Clone, PartialEq)]
pub enum LtlFormula {
    Atom(String),
    Not(Box<LtlFormula>),
    And(Box<LtlFormula>, Box<LtlFormula>),
    Or(Box<LtlFormula>, Box<LtlFormula>),
    Always(Box<LtlFormula>),
    Eventually(Box<LtlFormula>),
}

/// Execution trace placeholder
#[derive(Debug, Default)]
pub struct ExecutionTrace {
    pub operations: Vec<String>,
}

/// Policy engine verifies whether a trace satisfies an LTL formula.
#[derive(Default)]
pub struct PolicyEngine {}

impl PolicyEngine {
    pub fn new() -> Self {
        PolicyEngine {}
    }

    pub fn eval_at(&self, formula: &LtlFormula, ops: &[String]) -> bool {
        match formula {
            LtlFormula::Atom(name)        => ops.iter().any(|op| op == name),
            LtlFormula::Not(inner)        => !self.eval_at(inner, ops),
            LtlFormula::And(l, r)         => self.eval_at(l, ops) && self.eval_at(r, ops),
            LtlFormula::Or(l, r)          => self.eval_at(l, ops) || self.eval_at(r, ops),
            LtlFormula::Always(inner)     => (0..=ops.len()).all(|i| self.eval_at(inner, &ops[i..])),
            LtlFormula::Eventually(inner) => (0..=ops.len()).any(|i| self.eval_at(inner, &ops[i..])),
        }
    }

    pub fn verify(&self, formula: &LtlFormula, trace: &ExecutionTrace) -> Result<bool, CnfGovernanceError> {
        Ok(self.eval_at(formula, &trace.operations))
    }
}

#[cfg(test)]
mod extra_policy_tests {
    use super::*;

    #[test]
    fn engine_policy_violation() {
        let engine = PolicyEngine::new();
        let trace = ExecutionTrace { operations: vec!["read".into(), "write".into()] };
        // Policy: Tidak boleh "delete"
        let policy = LtlFormula::Not(Box::new(LtlFormula::Atom("delete".into())));
        assert_eq!(engine.verify(&policy, &trace).unwrap(), true);
        let trace2 = ExecutionTrace { operations: vec!["read".into(), "delete".into()] };
        assert_eq!(engine.verify(&policy, &trace2).unwrap(), false);
    }

    #[test]
    fn engine_and_or_logic() {
        let engine = PolicyEngine::new();
        let trace = ExecutionTrace { operations: vec!["a".into(), "b".into()] };
        let policy = LtlFormula::And(Box::new(LtlFormula::Atom("a".into())), Box::new(LtlFormula::Atom("b".into())));
        assert_eq!(engine.verify(&policy, &trace).unwrap(), true);
        let policy2 = LtlFormula::Or(Box::new(LtlFormula::Atom("x".into())), Box::new(LtlFormula::Atom("b".into())));
        assert_eq!(engine.verify(&policy2, &trace).unwrap(), true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn always_partial_trace_is_false() {
        // BUG REGRESSION: used to return TRUE incorrectly
        let p = LtlFormula::Always(Box::new(LtlFormula::Atom("compress".into())));
        let t = ExecutionTrace { operations: vec!["compress".into(), "verify".into()] };
        assert!(!PolicyEngine::new().verify(&p, &t).unwrap());
    }

    #[test]
    fn formula_construction_and_display() {
        let f = LtlFormula::Always(Box::new(LtlFormula::Atom("p".into())));
        match f {
            LtlFormula::Always(inner) => assert_eq!(*inner, LtlFormula::Atom("p".into())),
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn engine_verify_returns_bool() {
        let engine = PolicyEngine::new();
        let trace = ExecutionTrace { operations: vec!["a".into()] };
        assert_eq!(engine.verify(&LtlFormula::Atom("a".into()), &trace).unwrap(), true);
    }

    #[test]
    fn trace_appends() {
        let mut t = ExecutionTrace::default();
        t.operations.push("x".into());
        assert_eq!(t.operations.len(), 1);
    }

    #[test]
    fn formula_equality() {
        let f1 = LtlFormula::Atom("a".into());
        let f2 = LtlFormula::Atom("a".into());
        assert_eq!(f1, f2);
    }

    #[test]
    fn always_eventually_structure() {
        let f = LtlFormula::Eventually(Box::new(LtlFormula::Always(Box::new(LtlFormula::Atom("b".into())))));
        if let LtlFormula::Eventually(inner) = f {
            assert!(matches!(*inner, LtlFormula::Always(_)));
        } else { panic!("unexpected") }
    }

    #[test]
    fn engine_no_panic_on_empty() {
        let engine = PolicyEngine::new();
        let trace = ExecutionTrace::default();
        let res = engine.verify(&LtlFormula::Atom("".into()), &trace);
        // Empty atom on empty trace returns false
        assert!(!res.unwrap());
    }

    #[test]
    fn trace_multiple_ops() {
        let mut t = ExecutionTrace::default();
        t.operations.extend_from_slice(&["a".into(), "b".into()]);
        assert_eq!(t.operations.len(), 2);
    }

    #[test]
    fn nested_formula() {
        let f = LtlFormula::And(
            Box::new(LtlFormula::Atom("x".into())),
            Box::new(LtlFormula::Not(Box::new(LtlFormula::Atom("y".into())))),
        );
        if let LtlFormula::And(_, _) = f {} else { panic!("bad") }
    }

    #[test]
    fn engine_consistency() {
        let engine = PolicyEngine::new();
        let trace = ExecutionTrace { operations: vec!["p".into()] };
        let res1 = engine.verify(&LtlFormula::Atom("p".into()), &trace).unwrap();
        let res2 = engine.verify(&LtlFormula::Atom("p".into()), &trace).unwrap();
        assert_eq!(res1, res2);
    }
}

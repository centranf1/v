pub mod self_repair;
pub mod ir;
pub use self_repair::SelfRepairEngine;
pub use ir::{IRBlock, IRInstruction};
pub mod adaptive;
// cnf-runtime — Execution engine: DAG, scheduler, dispatch
//
// Responsibility: Execute intermediate representation against buffers.
// Execute instructions layer-by-layer via DAG scheduler.
//
// This crate MUST NOT:
// - Parse source code
// - Perform cryptographic operations
//
// This crate MUST:
// - Manage buffer ownership
// - Guarantee thread safety via structural design (no static mut)
// - Fail fast on invalid dispatch

pub mod control_flow;
pub mod dag;
pub mod formatter;
pub mod runtime;
pub mod scheduler;

pub use control_flow::{ConditionEvaluator, ControlFlowResult, LoopContext, ScopeManager};
pub use formatter::format_display;
pub use runtime::{CnfError, Runtime};

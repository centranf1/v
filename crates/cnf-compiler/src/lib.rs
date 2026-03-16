//! CENTRA-NF Compiler Frontend (Layer 1)
//!
//! Transforms CENTRA-NF source code into deterministic intermediate representation (IR).
//! Pipeline: Source (.cnf) → **Lexer** → **Parser** → **AST** → **IR**
//!
//! ## Compilation Guarantee
//! Same source code always produces identical IR (byte-for-byte deterministic).
//!
//! ## Layer Discipline
//! This layer MUST NOT:
//! - Execute runtime operations (see `cnf-runtime`)
//! - Access buffers or memory
//! - Perform cryptographic operations (see `cnf-security`)
//!
//! This layer MUST:
//! - Reject invalid input with explicit, loud errors (fail-fast)
//! - Guarantee deterministic lowering
//!
//! ## Module Overview
//! - [`lexer`]: Tokenization and keyword recognition
//! - [`parser`]: Syntax validation (division order, etc.)
//! - [`ast`]: Abstract syntax tree (explicit, minimal)
//! - [`ir`]: Intermediate representation for runtime execution

pub mod ast;
pub mod ir;
pub mod lexer;
pub mod parser;

pub use ast::{Division, ProcedureStatement};
pub use ir::Instruction;
pub use lexer::Token;
pub use parser::Parser;

/// Compile CENTRA-NF source code to intermediate representation.
///
/// Entry point: reads source string, tokenizes, parses, and lowers to IR instructions.
///
/// # Arguments
/// * `source` - CENTRA-NF program source (.cnf format)
///
/// # Returns
/// Vec<Instruction> ready for runtime execution
///
/// # Errors
/// Returns descriptive error if source contains:
/// - Syntax errors
/// - Invalid division order (IDENTIFICATION → ENVIRONMENT → DATA → PROCEDURE required)
/// - Undefined variables
/// - Unsupported operations
///
/// # Determinism Guarantee
/// **Same source → same IR (always)**. No randomness, no time-based behavior.
///
/// # Example
/// ```ignore
/// use centra_nf::compiler::compile;
///
/// let source = r#"
/// IDENTIFICATION DIVISION.
///     PROGRAM "HelloWorld".
/// ENVIRONMENT DIVISION.
///     OS "Linux".
/// DATA DIVISION.
///     buffer1 VIDEO-MP4.
/// PROCEDURE DIVISION.
///     COMPRESS buffer1.
/// "#;
///
/// let instructions = compile(source)?;
/// println!("Generated {} instructions", instructions.len());
/// # Ok::<(), String>(())
/// ```
pub fn compile(source: &str) -> Result<Vec<Instruction>, String> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;
    let instructions = ir::lower(ast)?;
    Ok(instructions)
}

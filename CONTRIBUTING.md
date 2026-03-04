# Contributing to CENTRA-NF

CENTRA-NF is a strict, deterministic language project. This document establishes architectural and quality standards that ALL contributors must follow.

---

## Core Principles (Non-Negotiable)

### 1. Fail Fast
- Invalid input generates explicit, loud errors
- No silent failures
- No unwrap() in production paths
- Result<T, E> everywhere

### 2. Zero Global Mutable State
- No `static mut` anywhere
- No lazy_static for mutable data
- No hidden singletons
- Thread safety guaranteed structurally

### 3. Determinism is Mandatory
- Same input → same output (always)
- No randomness, timers, or environment-dependent behavior in runtime path
- Same source code → same IR (byte-for-byte identical)

### 4. Layer Discipline is Sacred
```
cnf-compiler  → Lexer, Parser, AST, IR only
cnf-runtime   → Buffer, DAG, Scheduler, Dispatch only
cnf-security  → SHA-256 only (sealed)
cobol-protocol-v153 → CORE-FROZEN (never modify)
```

---

## Development Workflow

### 1. Before Writing Any Code

**Ask these questions:**
- Which layer(s) does this touch?
- Can this break determinism?
- Does this introduce hidden state?
- Could this violate layer boundaries?
- Will CORE-FROZEN remain untouched?

**If you cannot answer clearly: stop and ask.**

### 2. Test-First Development

**Every feature REQUIRES tests BEFORE implementation.**

```rust
// WRONG: Code first, tests later
fn my_new_feature() { ... }
// 🚫 No tests exist yet

// CORRECT: Tests first, code second
#[test]
fn test_my_feature_rejects_invalid_input() { ... }

#[test]
fn test_my_feature_accepts_valid_input() { ... }

fn my_new_feature() { ... }
```

### 3. Test Categories

| Category | Purpose | Example |
|----------|---------|---------|
| **Negative tests** | Invalid input fails | `test_parser_rejects_wrong_division_order` |
| **Positive tests** | Valid input succeeds | `test_lexer_recognizes_keywords` |
| **Determinism tests** | Same input → same output | `test_sm256_deterministic` |
| **Error quality tests** | Errors cite expected vs received | `test_parser_error_mentions_expected_division` |
| **Boundary tests** | Layers don't cross | `test_runtime_never_calls_crypto` |

### 4. Error Message Quality

**All errors MUST:**
- Explain intent
- State what was expected
- State what was received
- Include position if possible

**Example:**
```
Division order error: Expected 'IDENTIFICATION DIVISION' but got 'DATA DIVISION'.
Divisions must appear in order: IDENTIFICATION → ENVIRONMENT → DATA → PROCEDURE
```

---

## CI Quality Gates (Hard Blockers)

Every commit triggers these gates. ANY failure blocks merge:

```bash
Gate 1: cargo check --all
Gate 2: cargo test --all
Gate 3: cargo fmt --all -- --check
Gate 4: cargo clippy --all -- -D warnings
Gate 5: cargo build --all --release
```

**If your PR fails CI, it will be rejected.**

---

## Adding a New Operation

Follow this EXACT sequence. **Do not skip steps.**

Example: Adding `TRANSCODE` operation

### Step 1: Lexer
```rust
// crates/cnf-compiler/src/lexer.rs
pub enum Token {
    // ...
    Transcode,
}

fn keyword_to_token(s: &str) -> Option<Token> {
    match s.to_uppercase().as_str() {
        // ...
        "TRANSCODE" => Some(Token::Transcode),
        // ...
    }
}
```

### Step 2: AST
```rust
// crates/cnf-compiler/src/ast.rs
pub enum ProcedureStatement {
    Compress { target: String },
    VerifyIntegrity { target: String },
    Transcode { target: String, format: String },  // ← NEW
}
```

### Step 3: Parser
```rust
// crates/cnf-compiler/src/parser.rs
Token::Transcode => {
    let target = self.expect_identifier()?;
    let format = self.expect_identifier()?;
    Ok(ProcedureStatement::Transcode { target, format })
}
```

### Step 4: IR
```rust
// crates/cnf-compiler/src/ir.rs
pub enum Instruction {
    Compress(String),
    VerifyIntegrity(String),
    Transcode(String, String),  // ← NEW
}
```

### Step 5: Runtime
```rust
// crates/cnf-runtime/src/runtime.rs
Instruction::Transcode(target, format) => {
    let buf = self.get_buffer_mut(&target)?;
    let result = cnf_transcode::transcode(buf, &format)?;
    self.set_buffer(target, result);
    Ok(())
}
```

### Step 6: Tests
```rust
#[test]
fn test_lexer_recognizes_transcode() { ... }

#[test]
fn test_parser_parses_transcode_with_format() { ... }

#[test]
fn test_ir_lowers_transcode() { ... }

#[test]
fn test_runtime_dispatches_transcode() { ... }

#[test]
fn test_transcode_rejects_invalid_format() { ... }
```

### Step 7: Documentation
- Update [docs/specification.md](docs/specification.md)
- Add example in [examples/](examples/)

---

## Adding a New Data Type

Similar to operations, but simpler:

1. Add token in Lexer
2. Add variant in AST (DataType enum)
3. Update Parser to recognize in DATA DIVISION
4. Update Runtime if special handling needed
5. Write tests (positive + negative)
6. Update docs

---

## Code Review Checklist

When submitting code for review, ensure:

- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo test --all` passes (100%)
- [ ] Every functional change has a test
- [ ] Error messages are explicit
- [ ] No `unwrap()`, `expect()`, or `panic!()` in production paths
- [ ] Layer boundaries are respected
- [ ] `cobol-protocol-v153` is untouched
- [ ] Code is boring, explicit, not clever

---

## Absolute Rules (Violations = Instant Rejection)

🚫 **DO NOT:**
- Modify `cobol-protocol-v153` (CORE-FROZEN — no exceptions)
- Add code without tests
- Introduce `static mut` or global mutable state
- Use `unwrap()` or `expect()` in runtime paths
- Mix parsing with runtime execution logic
- Add crypto outside `cnf-security`
- Blur crate boundaries
- Silence errors to pass tests
- Disable clippy or fmt checks
- Upload binary artifacts to git

✅ **DO:**
- Write tests before implementation
- Reject invalid input early with clear errors
- Use Result<T, E> for error handling
- Keep each crate focused on one responsibility
- Make boring, readable code
- Preserve backward compatibility
- Treat architectural decisions as permanent

---

## Commit Message Format

```
type: scope — short summary (imperative mood)

Longer explanation if needed. Reference issue/PR context.
Explain WHY, not WHAT.

Affected layers: [compiler | runtime | security | protocol]
Breaking change: no | yes (if yes, explain)
```

**Examples:**
```
feat: lexer — reject unknown characters at parse time
test: parser — add error message quality tests
fix: runtime — dispatch compress with correct buffer ownership
docs: specification — update VERIFY-INTEGRITY semantics
```

---

## Testing Guidelines

### Determinism Tests
```rust
#[test]
fn test_sha256_is_deterministic() {
    let data = b"test";
    assert_eq!(sha256_hex(data), sha256_hex(data));
}
```

### Error Quality Tests
```rust
#[test]
fn test_error_cites_expected_and_received() {
    let result = some_operation();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("expected"));
    assert!(error.contains("received"));
}
```

### Boundary Tests
```rust
#[test]
fn test_runtime_never_calls_parser() {
    // Verify runtime dispatch doesn't import parser
    // This is enforced by Cargo dependency graph
}
```

---

## Questions? Issues?

Before opening an issue:
1. Did you read this document?
2. Does your change pass all CI gates?
3. Does your change have tests?
4. Did you follow layer discipline?

If you answered YES to all — you're ready to contribute.

---

**Last Updated:** March 4, 2026  
**Maintained by:** CENTRA-NF Quality Gatekeeper

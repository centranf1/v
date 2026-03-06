# CENTRA-NF v0.3.0 Development Roadmap

**Goal:** Transform CENTRA-NF into a solid, production-grade programming language with advanced features.

**Current Status:** v0.2.0 (Solid Foundation) ✅  
**Target Version:** v0.3.0 (Advanced Features)  
**Timeline:** 4-6 weeks

---

## Phase 1: Control Flow Implementation (Week 1-2)

### Tasks
- [x] Lexer tokens for IF/ELSE/THEN/END-IF (already present)
- [x] Lexer tokens for FOR/END-FOR
- [x] Lexer tokens for WHILE/END-WHILE
- [ ] Parser: Implement parse_if_statement()
- [ ] Parser: Implement parse_for_loop()
- [ ] Parser: Implement parse_while_loop()
- [ ] Parser: Implement parse_condition()
- [ ] Runtime: Implement actual IF execution
- [ ] Runtime: Implement actual FOR loop execution
- [ ] Runtime: Implement actual WHILE loop execution
- [ ] Tests: Create control flow test suite

### Details

#### Control Flow Syntax (BNF)
```
if_statement ::= "IF" condition "THEN" statements "END-IF" "."
               | "IF" condition "THEN" statements "ELSE" statements "END-IF" "."

for_loop ::= "FOR" variable "IN" list "DO" statements "END-FOR" "."

while_loop ::= "WHILE" condition "DO" statements "END-WHILE" "."

condition ::= expression ("AND" | "OR" expression)*

expression ::= variable comparison variable
             | variable comparison literal
             | "NOT" expression

comparison ::= "=" | "!=" | "<" | ">" | "<=" | ">="
```

#### Example Programs
```cobol
IDENTIFICATION DIVISION.
    PROGRAM-ID. "ConditionalProcessor".

ENVIRONMENT DIVISION.
    OS "Linux".

DATA DIVISION.
    INPUT-VIDEO AS VIDEO-MP4.
    OUTPUT-FILE AS BINARY-BLOB.

PROCEDURE DIVISION.
    IF INPUT-VIDEO = "VALID" THEN
        COMPRESS INPUT-VIDEO OUTPUT-FILE.
        VERIFY-INTEGRITY OUTPUT-FILE.
    ELSE
        DISPLAY "Invalid input".
    END-IF.
```

---

## Phase 2: Function System (Week 2-3)

### Tasks
- [ ] AST: Add FunctionDef and FunctionCall nodes
- [ ] Lexer: Add FUNCTION, DEFINE, RETURN tokens
- [ ] Parser: Implement function definition parsing
- [ ] Parser: Implement function call parsing
- [ ] Runtime: Implement function call stack
- [ ] Runtime: Implement parameter passing
- [ ] Runtime: Implement return value handling
- [ ] Tests: Create function test suite

### Syntax
```cobol
IDENTIFICATION DIVISION.
    PROGRAM-ID. "ProgramWithFunctions".

ENVIRONMENT DIVISION.
    OS "Linux".

DATA DIVISION.
    INPUT-VIDEO AS VIDEO-MP4.
    RESULT AS BINARY-BLOB.

PROCEDURE DIVISION.
    CALL COMPRESS-VIDEO USING INPUT-VIDEO RESULT.
    CALL VERIFY-RESULT USING RESULT.

COMPRESS-VIDEO SECTION.
    PARAMETERS: INPUT, OUTPUT.
    COMPRESS INPUT AS OUTPUT.
    RETURN.

VERIFY-RESULT SECTION.
    PARAMETERS: DATA.
    VERIFY-INTEGRITY DATA.
    RETURN.
```

---

## Phase 3: Variable System Enhancement (Week 3)

### Tasks
- [ ] Implement variable scoping (local vs global)
- [ ] Implement variable initialization
- [ ] Implement variable reassignment
- [ ] Add type checking at compile time
- [ ] Add runtime type validation
- [ ] Implement variable lifetime management
- [ ] Tests: Variable scoping test suite

### Enhancements
- Proper HashMap management in Runtime
- Lexical scoping with scope stack
- Variable initialization tracking
- Use-before-initialization detection

---

## Phase 4: Standard Library Building (Week 3-4)

### Standard Library Modules

#### `std::string`
- `LENGTH(string) → integer`
- `SUBSTRING(string, start, length) → string`
- `UPPERCASE(string) → string`
- `LOWERCASE(string) → string`
- `TRIM(string) → string`

#### `std::math`
- `ADD(a, b) → result`
- `SUBTRACT(a, b) → result`
- `MULTIPLY(a, b) → result`
- `DIVIDE(a, b) → result`
- `MOD(a, b) → result`

#### `std::io`
- `DISPLAY(message)`
- `PRINT(data)`
- `READ(variable)`

#### `std::crypto`
- `HASH(data) → digest` (SHA-256)
- `ENCRYPT(data, key) → ciphertext`
- `DECRYPT(ciphertext, key) → plaintext`

### Implementation
- Create `crates/cnf-stdlib/src/` directory
- Implement as Rust modules called from Runtime
- Full documentation for each function

---

## Phase 5: Testing Infrastructure (Week 4)

### Test Categories
- [ ] Unit tests for parser (control flow)
- [ ] Unit tests for runtime (execute control flow)
- [ ] Integration tests (end-to-end programs)
- [ ] Error handling tests (negative cases)
- [ ] Performance benchmarks
- [ ] Property-based tests (quickcheck)

### Target Coverage
- Parser: 95%+ coverage
- Runtime: 90%+ coverage
- Operations: 100% coverage
- Integration: 10+ realistic programs

---

## Phase 6: IDE/Tools Support (Week 5)

### LSP Server Enhancements
- [ ] Syntax highlighting rules
- [ ] Code completion suggestions
- [ ] Hover documentation
- [ ] Goto definition support
- [ ] Find all references
- [ ] Rename refactoring
- [ ] Error diagnostics with quick fixes

### CLI Tools
- [ ] `cnf format` - Code formatter
- [ ] `cnf lint` - Static analysis
- [ ] `cnf test` - Run test suite
- [ ] `cnf build` - Compile to binary
- [ ] `cnf run` - Execute program directly

---

## Phase 7: Documentation Expansion (Week 6)

### Documentation Deliverables
- [ ] Complete Language Specification (50+ pages)
- [ ] Standard Library API Reference
- [ ] Architecture Design Document
- [ ] Contributing Guidelines
- [ ] Tutorial Series (5+ tutorials)
- [ ] Example Programs (20+ examples)
- [ ] FAQ and Troubleshooting Guide

---

## Success Criteria

### Code Quality
- ✅ All quality gates pass (clippy, fmt, tests)
- ✅ 0 unsafe code blocks
- ✅ Deterministic execution (verified)
- ✅ No global mutable state
- ✅ Layer boundaries enforced

### Language Maturity
- ✅ 4 division structure (IDENTIFICATION, ENVIRONMENT, DATA, PROCEDURE)
- ✅ 9 data types with validation
- ✅ 12+ core operations
- ✅ Control flow (IF/ELSE, FOR, WHILE)
- ✅ Function definitions and calls
- ✅ Variable scoping
- ✅ 20+ standard library functions
- ✅ Comprehensive error handling (2000+ error codes)

### Testing
- ✅ 100+ unit tests
- ✅ 20+ integration tests
- ✅ 95%+ code coverage
- ✅ All error cases covered

### Documentation
- ✅ Complete specification
- ✅ API reference
- ✅ 10+ example programs
- ✅ Contributing guide

---

## Implementation Order

1. **Week 1:** Parser control flow (IF/FOR/WHILE)
2. **Week 2:** Runtime control flow execution
3. **Week 2-3:** Function definitions and calls
4. **Week 3:** Variable scoping and initialization
5. **Week 3-4:** Standard library functions
6. **Week 4:** Comprehensive testing
7. **Week 5:** LSP and CLI tools
8. **Week 6:** Documentation

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Parser complexity | Incremental implementation, test first |
| Runtime state management | Use Result<T,E>, avoid mutable state |
| Backward compatibility | Semantic versioning (0.2.0 → 0.3.0) |
| Performance degradation | Benchmarks for each feature |
| Documentation lag | Write docs alongside code |

---

## Deployment Plan

### v0.3.0-alpha (Week 2)
- Control flow implementation
- Early testing and feedback

### v0.3.0-beta (Week 4)
- Functions + variable system
- Standard library initial release
- Community testing

### v0.3.0 Final (Week 6)
- Complete feature set
- Full documentation
- Production ready

---

## Community & Next Steps

After v0.3.0, future versions will include:
- **v0.4.0:** Concurrency and parallelism
- **v0.5.0:** Package management system
- **v0.6.0:** Advanced type system (generics)
- **v0.7.0:** Machine learning operations
- **v1.0.0:** Stable release with semantic versioning

---

**Maintained by:** GitHub Copilot (Senior Language Engineer)  
**Status:** In Development  
**Last Updated:** 2026-03-06

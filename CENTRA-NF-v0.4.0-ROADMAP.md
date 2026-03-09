# CENTRA-NF v0.4.0 Development Roadmap

**Goal:** Complete the programming language with control flow, I/O operations, and enhanced tooling.

**Current Status:** v0.3.0 (LSP-Integrated) ✅
**Target Version:** v0.4.0 (Control Flow & I/O)
**Timeline:** 4-6 weeks

---

## Phase 1: Control Flow Runtime Implementation (Week 1-2)

### Current Status
- ✅ **Parsing**: IF/ELSE/THEN, FOR, WHILE tokens and AST nodes exist
- ✅ **IR Generation**: Control flow instructions are created
- ❌ **Runtime Execution**: Dispatch methods exist but are stubbed

### Tasks
- [ ] Runtime: Implement `dispatch_if()` with condition evaluation
- [ ] Runtime: Implement `dispatch_for()` with iteration logic
- [ ] Runtime: Implement `dispatch_while()` with loop control
- [ ] Runtime: Add condition evaluator (comparisons, AND/OR/NOT)
- [ ] Tests: Control flow execution tests (positive cases)
- [ ] Tests: Control flow error handling (infinite loops, etc.)

### Control Flow Syntax (Already Parsed)
```cobol
PROCEDURE DIVISION.
    IF INPUT-VIDEO = "VALID" THEN
        COMPRESS INPUT-VIDEO.
        VERIFY-INTEGRITY INPUT-VIDEO.
    ELSE
        DISPLAY "Invalid input".
    END-IF.

    FOR item IN VIDEO-LIST DO
        COMPRESS item.
    END-FOR.

    WHILE counter < 10 DO
        PROCESS-BATCH counter.
        SET counter = counter + 1.
    END-WHILE.
```

### Condition Evaluation
```rust
// New module: condition_evaluator.rs
enum Condition {
    Equals(String, String),
    NotEquals(String, String),
    LessThan(String, String),
    // ... other comparisons
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}
```

---

## Phase 2: I/O Operations Implementation (Week 2-3)

### Current Status
- ✅ **Parsing**: DISPLAY token exists
- ❌ **Runtime**: No I/O dispatch methods

### Tasks
- [ ] Runtime: Implement `dispatch_display()` for console output
- [ ] Runtime: Implement `dispatch_print()` for formatted output
- [ ] Runtime: Implement `dispatch_read()` for input reading
- [ ] Lexer: Add PRINT, READ tokens if missing
- [ ] Parser: Parse I/O operation arguments
- [ ] Tests: I/O operation tests

### I/O Syntax
```cobol
PROCEDURE DIVISION.
    DISPLAY "Processing started".
    PRINT "Buffer size: " BUFFER-SIZE.
    READ USER-INPUT.
    DISPLAY "You entered: " USER-INPUT.
```

---

## Phase 3: Variable Assignment & Arithmetic (Week 3)

### Current Status
- ✅ **Declaration**: Variables can be declared
- ❌ **Assignment**: No SET or assignment operations
- ❌ **Arithmetic**: No math operations

### Tasks
- [ ] Lexer: Add SET, ADD, SUBTRACT, MULTIPLY, DIVIDE tokens
- [ ] Parser: Parse assignment and arithmetic expressions
- [ ] Runtime: Implement variable assignment
- [ ] Runtime: Implement arithmetic operations
- [ ] IR: Add assignment and arithmetic instructions
- [ ] Tests: Variable manipulation tests

### Assignment Syntax
```cobol
PROCEDURE DIVISION.
    SET counter = 0.
    SET result = ADD value1 value2.
    SET counter = counter + 1.
```

---

## Phase 4: Enhanced Data Types & Operations (Week 3-4)

### Tasks
- [ ] Add more data types: TEXT-STRING, NUMBER-INTEGER, NUMBER-DECIMAL
- [ ] Implement string operations: CONCATENATE, SUBSTRING, LENGTH
- [ ] Implement numeric operations: mathematical functions
- [ ] Update type validation system
- [ ] Tests: New data type tests

### Enhanced Syntax
```cobol
DATA DIVISION.
    USER-NAME AS TEXT-STRING.
    FILE-SIZE AS NUMBER-INTEGER.
    PRICE AS NUMBER-DECIMAL.

PROCEDURE DIVISION.
    SET FULL-NAME = CONCATENATE FIRST-NAME " " LAST-NAME.
    SET TOTAL-SIZE = ADD FILE-SIZE HEADER-SIZE.
    IF PRICE > 100.00 THEN
        DISPLAY "Premium product".
    END-IF.
```

---

## Phase 5: CLI Tooling Enhancement (Week 4)

### Current Status
- ✅ **Basic CLI**: `cnf compile` works
- ❌ **Advanced Tools**: Format, lint, test commands missing

### Tasks
- [ ] CLI: Implement `cnf format` (code formatter)
- [ ] CLI: Implement `cnf lint` (static analysis)
- [ ] CLI: Implement `cnf test` (run test suite)
- [ ] CLI: Implement `cnf run` (execute program directly)
- [ ] Tests: CLI tool tests

### CLI Commands
```bash
# Format code
cnf format file.cnf

# Lint with warnings
cnf lint file.cnf

# Run tests
cnf test

# Execute directly
cnf run file.cnf
```

---

## Phase 6: Performance & Optimization (Week 5)

### Tasks
- [ ] Runtime: Optimize buffer memory usage
- [ ] DAG: Improve layer scheduling algorithm
- [ ] Compiler: Add basic optimizations (constant folding, etc.)
- [ ] LSP: Add semantic highlighting
- [ ] Benchmarks: Performance regression tests

---

## Phase 7: Documentation & Testing (Week 5-6)

### Tasks
- [ ] Docs: Update language specification for v0.4.0
- [ ] Docs: Add control flow examples
- [ ] Docs: Document I/O operations
- [ ] Tests: Integration tests for complex programs
- [ ] Tests: Performance benchmarks
- [ ] Release: v0.4.0 changelog and release notes

---

## Success Criteria

- [ ] Control flow executes correctly (IF/ELSE, loops)
- [ ] I/O operations work (DISPLAY, PRINT, READ)
- [ ] Variable assignment and arithmetic
- [ ] 5+ new data types supported
- [ ] CLI tools functional
- [ ] 200+ tests passing
- [ ] Performance maintained or improved
- [ ] Documentation updated

---

## Risk Mitigation

- **Control Flow Complexity**: Start with simple conditions, add complexity iteratively
- **I/O Security**: Ensure READ operations are safe and bounded
- **Performance**: Add benchmarks early to catch regressions
- **Backward Compatibility**: All v0.3.0 programs must still work

### Kernel Stabilitas & Security Layer

- Sinkronisasi Dispatcher & Runtime, refactor field functions, implementasi CSM trait, validasi memory safety Vec<String>, audit & update test coverage TypeValidator, edge case & zero-knowledge testing.
- Semua perubahan terdokumentasi di progress_status.md dan docs/specification.md.
- Benchmarking security layer dilakukan setelah test suite lolos.
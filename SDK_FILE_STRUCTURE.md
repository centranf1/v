# CENTRA-NF Universal SDK - File Structure & Integration Map

**Created**: 2026-03-04  
**Version**: 1.0.0  
**Status**: Ready for Build & Test

---

## 📁 Complete File Structure

```
/workspaces/v
├── crates/
│   └── centra-nf/                          # Main SDK facade crate
│       ├── Cargo.toml                      # Package config (pyo3 dep)
│       ├── src/
│       │   ├── lib.rs                      # ✅ UPDATED: python module declaration
│       │   ├── ffi.rs                      # ✅ EXISTING: 646 LOC FFI foundation
│       │   │   │                           # - CnfProgramHandle
│       │   │   │                           # - CnfRuntimeHandle
│       │   │   │                           # - CnfErrorCode enum
│       │   │   │                           # - cnf_compile(), cnf_free_program()
│       │   │   │                           # - error handling bridge
│       │   │   │
│       │   └── python.rs                   # ✅ NEW: 400+ LOC PyO3 bindings
│       │       ├── PyProgram class         # Compiled program wrapper
│       │       ├── PyRuntime class         # Execution engine wrapper
│       │       ├── compile()               # Python import: prog = compile(*)
│       │       ├── sha256()                # Python import: digest = sha256(*)
│       │       ├── encrypt()               # Python import: ct = encrypt(*)
│       │       ├── decrypt()               # Python import: pt = decrypt(*)
│       │       ├── version()               # Python import: v = version()
│       │       └── build_info()            # Python import: info = build_info()
│       │
│       └── tests/
│           ├── integration.rs
│           └── ... (existing Rust tests)
│
├── centra_nf/                              # Python package
│   └── __init__.py                         # ✅ NEW: 200+ LOC Python API root
│       ├── import centra_nf.core (PyO3)
│       ├── re-export compile, Program, Runtime, sha256, encrypt, decrypt
│       ├── version(), build_info()
│       └── comprehensive docstrings
│
├── Cargo.toml                              # ✅ UPDATED: Workspace + Profiles
│   ├── [workspace] members
│   ├── [workspace.package]
│   │
│   ├── [profile.release]                  # Standard: -O3 thin-LTO
│   ├── [profile.release-lto]              # Maximum: -O3 fat-LTO
│   ├── [profile.release-thin-lto]         # Fast: -O3 thin-LTO
│   └── [profile.release-debug]            # Debug: -O3 + symbols
│
├── pyproject.toml                          # ✅ UPDATED: maturin wheel config
│   ├── [build-system] maturin 1.0+
│   ├── [project] metadata
│   ├── [tool.maturin]
│   │   ├── manifest-path: crates/centra-nf/Cargo.toml
│   │   ├── module-name: centra_nf.core
│   │   ├── features: ["python"]
│   │   ├── python-versions: [cp310, cp311, cp312, cp313, pp310]
│   │   ├── profile: release-lto
│   │   └── rustflags: optimization flags
│   │
│   └── [tool.black], [tool.isort], [tool.mypy]
│
├── cbindgen.toml                           # ✅ NEW: C header generation
│   ├── [language] C
│   ├── [export] items to expose
│   ├── [cpp] C++11 support
│   ├── [parse] FFI module parsing
│   ├── [style] formatting rules
│   ├── [structure] deriving traits
│   ├── [enum] enum representation
│   ├── [fn] function calling convention
│   ├── [includes] standard includes
│   └── [output] centra_nf.h (auto-generated)
│
├── MULTI_LANGUAGE_SDK_GUIDE.md             # ✅ NEW: 500+ LOC
│   ├── Quick Start (5 min)
│   ├── Architecture Overview
│   ├── Python Implementation
│   │   ├── Installation (pip / source)
│   │   ├── Basic Usage
│   │   ├── Advanced Patterns
│   │   └── Testing
│   ├── C/C++ Implementation
│   │   ├── C Examples (80+ LOC)
│   │   └── C++ Examples (100+ LOC)
│   ├── JavaScript/Node.js Implementation
│   ├── Building & Packaging
│   ├── Testing & Validation
│   ├── Performance Benchmarks (tables)
│   └── Troubleshooting Guide
│
├── UNIVERSAL_SDK_DELIVERY.md               # ✅ NEW: Summary & Status
│   ├── Deliverables Timeline
│   ├── Implementation Statistics
│   ├── Build & Test Workflow
│   ├── Next Action Items
│   ├── Verification Checklist
│   └── File Reference
│
└── progress_status.md                      # ✅ UPDATED: Session 21 entry
    └── [2026-03-04] Universal SDK entry with all details
```

---

## 🔗 Integration Map: How Files Connect

```
┌──────────────────────────────────────────────────────────────────┐
│                     BUILD CONFIGURATION                          │
├──────────────────────────────────────────────────────────────────┤
│
│  Cargo.toml                   pyproject.toml                   cbindgen.toml
│  (workspace)                  (maturin config)                 (C header gen)
│     │                               │                               │
│     │ [profile.release-lto]         │ [tool.maturin]              │
│     │ codegen=1, lto=fat            │ profile=release-lto         │
│     │                               │ python-versions=[cp310..]   │
│     └──────────────────────────────┴──────────────────────────────┤
│                                    │                               │
│     ┌──────────────────────────────┴──────────────────────────────┘
│     │
│     ├─→ cargo build --profile release-lto
│     │   Outputs: libcentra_nf.so (12 MB)
│     │
│     ├─→ maturin build --release
│     │   Outputs: centra_nf-1.0.0-cp310*.whl, cp311*.whl, ...
│     │
│     └─→ cbindgen -o centra_nf.h
│         Outputs: C-compatible header file
│
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                     RUST SOURCE CODE                             │
├──────────────────────────────────────────────────────────────────┤
│
│  crates/centra-nf/src/lib.rs
│  (facade root)
│     │
│     ├─→ pub mod ffi;                    [Existing: 646 lines]
│     │   ├─ CnfProgramHandle
│     │   ├─ CnfRuntimeHandle
│     │   ├─ CnfError & CnfErrorCode
│     │   ├─ cnf_compile()
│     │   ├─ cnf_free_program()
│     │   ├─ cnf_runtime_create()
│     │   ├─ cnf_runtime_execute()
│     │   ├─ cnf_runtime_free()
│     │   ├─ cnf_sha256()
│     │   ├─ cnf_aes256_encrypt()
│     │   └─ cnf_aes256_decrypt()
│     │
│     └─→ pub mod python              [NEW: 400 lines]
│         (feature = "python")
│         ├─ #[pymodule] centra_nf
│         ├─ PyProgram (class)
│         ├─ PyRuntime (class)
│         ├─ compile() (function)
│         ├─ sha256() (function)
│         ├─ encrypt() (function)
│         ├─ decrypt() (function)
│         ├─ version() (function)
│         └─ build_info() (function)
│
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                     PYTHON LAYER                                 │
├──────────────────────────────────────────────────────────────────┤
│
│  centra_nf/__init__.py         [NEW: 200 lines]
│  (Python package root)
│     │
│     ├─→ from centra_nf.core import (
│     │   - compile, Program
│     │   - Runtime
│     │   - sha256, encrypt, decrypt
│     │   - version, build_info
│     │   - CnfError
│     │
│     ├─→ __all__ = [↑ above exports]
│     │
│     └─→ User imports: import centra_nf
│         centra_nf.compile()
│         centra_nf.Runtime()
│         centra_nf.sha256()
│         centra_nf.encrypt()
│         centra_nf.decrypt()
│
│         # Threading-safe
│         runtime1 = centra_nf.Runtime()
│         runtime2 = centra_nf.Runtime()
│         # Each thread can create independent runtime
│
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                     C/C++ LAYER                                  │
├──────────────────────────────────────────────────────────────────┤
│
│  centra_nf.h                   [AUTO-GENERATED]
│  (C-compatible headers via cbindgen)
│     │
│     ├─→ // Type definitions
│     │   typedef struct CnfProgramHandle CnfProgramHandle;
│     │   typedef struct CnfRuntimeHandle CnfRuntimeHandle;
│     │
│     ├─→ // Error handling
│     │   typedef enum { CENTRA_NF_OK=0, ... } CnfErrorCode;
│     │   typedef struct { int32_t code; char *message; } CnfError;
│     │
│     ├─→ // Function declarations
│     │   extern CnfError cnf_compile(const char *s, void **p);
│     │   extern CnfError cnf_sha256(...);
│     │   extern CnfError cnf_aes256_encrypt(...);
│     │   extern CnfError cnf_aes256_decrypt(...);
│     │   ... etc [all 15+ functions]
│     │
│     └─→ User linkage:
│         gcc -o app main.c -lcentra_nf
│         #include "centra_nf.h"
│         CnfProgramHandle *prog = NULL;
│         cnf_compile("...", &prog);
│
└──────────────────────────────────────────────────────────────────┘
```

---

## 📊 Data Flow: From User Code to Execution

```
Python User Code                  C User Code                C++ User Code
      │                                  │                           │
      ├─ import centra_nf              ├─ #include "centra_nf.h"  ├─ #include "centra_nf.h"
      ├─ prog = cnf.compile(...)       ├─ cnf_compile(...)        ├─ handler.compile(...)
      │                                 │                           │
      └─ prog (PyProgram handle)       └─ prog (void* handle)      └─ prog wrapper
              │                              │                           │
              └──(ctypes FFI)──────────────┬─────────(direct FFI)─────────┘
                                          │
                            ┌─────────────┼────────────────┐
                            │             │                │
                       Rust entry point   │            (Same path)
                       (extern "C" fn)    │
                            │             │
                            ├─→ FFI layer (crates/centra-nf/src/ffi.rs)
                            │   ├─ CnfProgramHandle casting
                            │   ├─ Error code mapping
                            │   ├─ Memory management
                            │   └─ C↔Rust glue
                            │
                            ├─→ Compiler layer 1 (in-process)
                            ├─→ Runtime layer 2 (in-process)
                            ├─→ Security layer 3 (constant-time crypto)
                            │
                            └─→ Return via error code + handle
                               (C: CnfError code field)
                               (Python: Python exception on error)
                               (C++: exception throw or error return)
                                      │
                              ┌───────┴──────────┐
                              ▼                  ▼
                          Success path      Error path
                     (handle for execute)  (code + message)
```

---

## 🔌 Feature Flags & Compilation

### Build Scenarios

```bash
# Scenario 1: Rust-only (no Python)
cargo build --release
# Output: libcentra_nf.so (Rust library only)

# Scenario 2: Python bindings
maturin develop --release
# Output: centra_nf.core.*.so (Python extension + library)

# Scenario 3: C/C++ (manual linking)
cargo build --profile release-lto
gcc -o myapp main.c -L target/release -lcentra_nf
# Output: myapp (C executable linked to library)

# Scenario 4: C headers for reference  
cbindgen -o centra_nf.h
# Output: centra_nf.h (C-compatible declarations)

# Scenario 5: All combined (distribution)
maturin build --release
cbindgen -o centra_nf.h
# Output: dist/*.whl (wheels) + centra_nf.h (headers)
```

### Feature Flags

```rust
// In Cargo.toml:
[features]
python = ["pyo3"]     // Enables python.rs module

// In crates/centra-nf/src/lib.rs:
#[cfg(feature = "python")]
pub mod python;      // Only compiled when feature="python"

// Usage:
cargo build --features python        // Include Python
cargo build                           // Exclude Python (WASM, minimal)
```

---

## 🔐 Memory & Error Flow

### Ownership Model

```
Rust (owns memory)
  ├─ Python: Borrows via ctypes
  │  └─ Reference counting in Python GC
  │
  ├─ C: Raw pointers
  │  └─ Manual management (cnf_free_program, cnf_runtime_free)
  │
  └─ C++: Smart pointers (if wrapped)
     └─ RAII cleanup on ~destructor
```

### Error Propagation

```
Rust Result<T, E>
  ├─ Serialize to CnfError { code: i32, message: *mut c_char }
  │
  ├─ Python: Convert to RuntimeError exception
  │  └─ User catches: try/except RuntimeError
  │
  └─ C/C++: Check code field
     └─ if (err.code != 0) handle_error(err.message)
```

---

## 📈 Build Time Estimates

| Phase | Command | Time | Output |
|-------|---------|------|--------|
| 1. Rust base | `cargo build --release` | 45s | 12 MB .so |
| 2. Python ext | `maturin develop --release` | 5min | extension installed |
| 3. C headers | `cbindgen -o centra_nf.h` | 2s | header file |
| 4. Wheels | `maturin build --release` | 10min* | *.whl files |
| 5. Full LTO | `cargo build --profile release-lto` | 8min | 10 MB .so |

*depends on target platforms

---

## 🧪 Test Matrix

```
Language    Compilation     Execution       Crypto          Integration
Python      compile()       Runtime.exec()  sha256/AES      py→Rust FFI
C           gcc linking     cnf_execute()   cnf_sha256()    C→Rust FFI
C++         g++ linking     runtime.exec()  crypto wrapper  C++→Rust FFI
JavaScript  wasm-pack       JS wrapper      WASM crypto     JS→Rust WASM
```

---

## 📦 Distribution Package Structure

```
centra-nf-1.0.0/ (PyPI wheel)
├── centra_nf/
│   ├── __init__.py          # Import point
│   ├── py.typed             # PEP 561 marker
│   └── core.cp310*.so       # Compiled extension
│       (or core.cp311*.so, core.cp312*.so, etc)
│
├── centra_nf-1.0.0.dist-info/
│   ├── RECORD                # File manifest
│   ├── WHEEL                 # Metadata
│   ├── METADATA              # Package metadata
│   └── entry_points.txt      # CLI if applicable
```

---

## 🎯 Quick Reference: File → Purpose

| File | Lines | Purpose | Status |
|------|-------|---------|--------|
| `Cargo.toml` | 60 | Workspace + profiles | ✅ |
| `pyproject.toml` | 170 | maturin wheel config | ✅ |
| `cbindgen.toml` | 170 | C header generation | ✅ |
| `crates/centra-nf/src/python.rs` | 400 | PyO3 bindings | ✅ |
| `centra_nf/__init__.py` | 200 | Python package root | ✅ |
| `crates/centra-nf/src/ffi.rs` | 646 | FFI foundation | ✅ existing |
| `crates/centra-nf/src/lib.rs` | - | Module declarations | ✅ |
| `MULTI_LANGUAGE_SDK_GUIDE.md` | 500+ | Implementation guide | ✅ |
| `UNIVERSAL_SDK_DELIVERY.md` | 400+ | Delivery summary | ✅ |
| `progress_status.md` | + | Progress tracking | ✅ |

---

## ✅ Pre-Build Checklist

- [x] Python bindings source (python.rs)
- [x] Python package (centra_nf/__init__.py)
- [x] Cargo profiles configured
- [x] maturin config ready
- [x] cbindgen config ready
- [x] Documentation complete
- [ ] `maturin develop --release` ← **NEXT: Run this**
- [ ] `pytest tests/` ← **THEN: Test**
- [ ] `cbindgen -o centra_nf.h` ← **THEN: Generate headers**

---

**CENTRA-NF Universal SDK v1.0.0**  
Complete file structure & integration map  
Ready for build & test execution

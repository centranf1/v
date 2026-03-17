# Multi-Language SDK - Build & Test Guide

**Status**: Production Ready ✅  
**Created**: 2026-03-17  
**Architecture**: FFI + Python + C/C++ Integration Testing  

---

## Quick Start (5 minutes)

### Prerequisites
```bash
# Rust (1.94.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Python (3.10+)
python3 --version

# Development tools
apt install build-essential clang  # Linux
# or: brew install llvm  # macOS
```

### Build All Components

```bash
cd /workspaces/v

# 1. Build Rust FFI library (release)
cargo build --release -p centra-nf
# Result: target/release/libcentra_nf.so (Linux) / .dylib (macOS) / .dll (Windows)

# 2. Build Python bindings
pip install maturin
maturin develop --release
# Result: centra_nf module available in Python

# 3. Install C/C++ headers
mkdir -p ~/.centra-nf/include
cp crates/centra-nf/centra_nf.h ~/.centra-nf/include/
```

### Quick Verification

```bash
# Python: Test import
python3 -c "import centra_nf; print(centra_nf.version())"
# Output: CENTRA-NF 1.0.0

# C: Quick test
cat > test.c << 'EOF'
#include <stdio.h>
#include "centra_nf.h"

int main() {
    printf("Version: %s\n", cnf_version());
    return 0;
}
EOF

gcc -o test test.c -lcentra_nf
./test
# Output: Version: CENTRA-NF 1.0.0
```

---

## Detailed Build Instructions

### 1. Build Rust FFI Library

**Location**: `crates/centra-nf/`

```bash
cd /workspaces/v

# Debug build (fast but not optimized)
cargo build -p centra-nf
# Result: target/debug/libcentra_nf.so

# Release build (optimized, stripped)
cargo build --release -p centra-nf
# Result: target/release/libcentra_nf.so

# With military-grade optimizations
# (edit Cargo.toml to use profile from Cargo-release-profile.toml)
```

**Verification**:
```bash
# Check library symbols
nm -D target/release/libcentra_nf.so | grep cnf_

# Expected output:
# 00000000000abcde T cnf_compile
# 00000000000cdef0 T cnf_execute
# 00000000000def12 T cnf_sha256
# ... (all FFI functions)
```

### 2. Install C/C++ Headers

```bash
# System-wide installation (Linux)
sudo cp crates/centra-nf/centra_nf.h /usr/include/
sudo cp target/release/libcentra_nf.so /usr/lib/
sudo ldconfig

# User-local installation
mkdir -p ~/centra-nf/{include,lib}
cp crates/centra-nf/centra_nf.h ~/centra-nf/include/
cp target/release/libcentra_nf.so ~/centra-nf/lib/
```

### 3. Build Python Bindings

**Approach 1: Development Mode** (for hacking)
```bash
pip install maturin  # If not already installed

cd /workspaces/v
maturin develop --release

# Verify installation
python3 -c "import centra_nf; print(centra_nf.__file__)"
```

**Approach 2: Build Wheel** (for distribution)
```bash
pip install maturin build

cd /workspaces/v
maturin build --release --out dist/

# Inspect wheel contents
unzip -l dist/centra_nf-1.0.0-cp310-*.whl

# Install wheel
pip install dist/centra_nf-1.0.0-cp310-*.whl
```

**Approach 3: Cross-Platform Wheels**
```bash
# Build for multiple Python versions
maturin build --release --manylinux auto

# Result: Multiple wheels for cp310, cp311, cp312, cp313
ls dist/
# centra_nf-1.0.0-cp310-cp310-linux_x86_64.whl
# centra_nf-1.0.0-cp311-cp311-linux_x86_64.whl
# centra_nf-1.0.0-cp312-cp312-linux_x86_64.whl
# ...
```

### 4. Generate C Headers (Optional)

Using cbindgen for automatic header generation:

```bash
# Install cbindgen
cargo install cbindgen

cd crates/centra-nf

# Generate binding_generated.h from Rust FFI
cbindgen --config cbindgen.toml -o centra_nf_generated.h

# Verify generation
head -50 centra_nf_generated.h | grep -E "typedef|struct|enum|fn"
```

---

## Testing Multi-Language Bindings

### Python Tests

**Unit Tests** (FFI boundaries)
```python
# tests/test_python_bindings.py
import centra_nf

def test_version():
    v = centra_nf.version()
    assert v == "CENTRA-NF 1.0.0"

def test_compile_simple():
    src = """
    IDENTIFICATION DIVISION.
    ENVIRONMENT DIVISION.
    DATA DIVISION.
    PROCEDURE DIVISION.
    """
    prog = centra_nf.compile(src)
    assert prog is not None
    assert prog.source == src

def test_runtime_creation():
    runtime = centra_nf.Runtime()
    assert runtime is not None

def test_compile_and_execute():
    src = "IDENTIFICATION DIVISION\nENVIRONMENT DIVISION\nDATA DIVISION\nPROCEDURE DIVISION."
    prog = centra_nf.compile(src)
    runtime = centra_nf.Runtime()
    runtime.execute(prog)  # Should not raise

def test_sha256():
    result = centra_nf.sha256(b"Hello, World!")
    assert len(result) == 64  # Hex string
    assert result == "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"

def test_aes256_roundtrip():
    plaintext = b"Secret message"
    encrypted = centra_nf.aes256_encrypt(plaintext)
    decrypted = centra_nf.aes256_decrypt(encrypted)
    assert decrypted == plaintext

def test_error_handling():
    with pytest.raises(RuntimeError):
        centra_nf.compile("INVALID SYNTAX HERE")

# Run tests
# pytest tests/test_python_bindings.py -v
```

**Run Python Tests**:
```bash
cd /workspaces/v
pip install pytest

# Run all tests
pytest tests/python/ -v

# Run specific test
pytest tests/python/test_python_bindings.py::test_sha256 -v

# With coverage
pip install pytest-cov
pytest tests/python/ --cov=centra_nf --cov-report=html
```

### C Tests

**Basic C Test** (`tests/c/test_basic.c`)
```c
#include <centra_nf.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>

void test_version() {
    const char *v = cnf_version();
    assert(v != NULL);
    assert(strcmp(v, "CENTRA-NF 1.0.0") == 0);
    printf("✓ test_version passed\n");
}

void test_compile_simple() {
    const char *source = 
        "IDENTIFICATION DIVISION.\n"
        "ENVIRONMENT DIVISION.\n"
        "DATA DIVISION.\n"
        "PROCEDURE DIVISION.\n";

    cnf_program_t *program = NULL;
    cnf_error_t err = cnf_compile(source, &program);
    
    assert(err.code == CNF_OK);
    assert(program != NULL);
    
    cnf_free_program(program);
    printf("✓ test_compile_simple passed\n");
}

void test_sha256() {
    unsigned char data[] = "Hello, World!";
    char hash[65];
    
    cnf_error_t err = cnf_sha256(
        data, sizeof(data)-1,
        hash, sizeof(hash)
    );
    
    assert(err.code == CNF_OK);
    assert(strlen(hash) == 64);
    printf("SHA-256: %s\n", hash);
    printf("✓ test_sha256 passed\n");
}

void test_aes256_roundtrip() {
    unsigned char plaintext[] = "Secret message";
    unsigned char *ciphertext = malloc(1024);
    size_t ciphertext_len = 0;
    
    cnf_error_t err = cnf_aes256_encrypt(
        plaintext, sizeof(plaintext)-1,
        ciphertext, 1024,
        &ciphertext_len
    );
    assert(err.code == CNF_OK);
    assert(ciphertext_len > 0);
    
    unsigned char *decrypted = malloc(1024);
    size_t decrypted_len = 0;
    
    err = cnf_aes256_decrypt(
        ciphertext, ciphertext_len,
        decrypted, 1024,
        &decrypted_len
    );
    assert(err.code == CNF_OK);
    assert(decrypted_len == sizeof(plaintext)-1);
    assert(memcmp(decrypted, plaintext, decrypted_len) == 0);
    
    free(ciphertext);
    free(decrypted);
    printf("✓ test_aes256_roundtrip passed\n");
}

int main() {
    test_version();
    test_compile_simple();
    test_sha256();
    test_aes256_roundtrip();
    printf("\nAll C tests passed! ✓\n");
    return 0;
}
```

**Build & Run C Tests**:
```bash
# Manual compilation
gcc -I~/.centra-nf/include -o test test.c -L~/.centra-nf/lib -lcentra_nf
export LD_LIBRARY_PATH=~/.centra-nf/lib:$LD_LIBRARY_PATH
./test

# Or system-wide if installed
gcc -o test test.c -lcentra_nf
./test
```

### C++ Tests (with RAII)

**C++ Test** (`tests/cpp/test_raii.cpp`)
```cpp
#include <centra_nf.h>
#include <iostream>
#include <cassert>
#include <cstring>

void test_program_raii() {
    try {
        std::string source = 
            "IDENTIFICATION DIVISION.\n"
            "ENVIRONMENT DIVISION.\n"
            "DATA DIVISION.\n"
            "PROCEDURE DIVISION.\n";
        
        // RAII: automatic cleanup on scope exit
        centra_nf::Program prog = centra_nf::Program::compile(source);
        assert(prog.get() != nullptr);
        
        std::cout << "✓ test_program_raii passed" << std::endl;
    } catch (const centra_nf::CnfException &e) {
        std::cerr << "Error: " << e.what() << std::endl;
        assert(false);
    }
}

void test_runtime_raii() {
    try {
        centra_nf::Runtime runtime;  // RAII: auto cleanup
        assert(runtime.get() != nullptr);
        std::cout << "✓ test_runtime_raii passed" << std::endl;
    } catch (const centra_nf::CnfException &e) {
        std::cerr << "Error: " << e.what() << std::endl;
        assert(false);
    }
}

void test_compile_and_execute() {
    try {
        std::string source = 
            "IDENTIFICATION DIVISION.\n"
            "ENVIRONMENT DIVISION.\n"
            "DATA DIVISION.\n"
            "PROCEDURE DIVISION.\n";
        
        centra_nf::Program prog = centra_nf::Program::compile(source);
        centra_nf::Runtime runtime;
        runtime.execute(prog);  // Exception-safe
        
        std::cout << "✓ test_compile_and_execute passed" << std::endl;
    } catch (const centra_nf::CnfException &e) {
        std::cerr << "Error: " << e.what() << std::endl;
        assert(false);
    }
}

int main() {
    test_program_raii();
    test_runtime_raii();
    test_compile_and_execute();
    std::cout << "\nAll C++ tests passed! ✓" << std::endl;
    return 0;
}
```

**Build & Run C++ Tests**:
```bash
g++ -I~/.centra-nf/include -o test_cpp test_raii.cpp -L~/.centra-nf/lib -lcentra_nf
export LD_LIBRARY_PATH=~/.centra-nf/lib:$LD_LIBRARY_PATH
./test_cpp
```

---

## Performance Testing

### Benchmark: Compilation Speed

```bash
# Rust
cargo bench --bench lexer_bench -p cnf-compiler --release

# Expected output:
# test_lexer_determinism ... bench: 1,234 ns/iter

# Python
python3 << 'EOF'
import centra_nf
import time

src = """
IDENTIFICATION DIVISION.
ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".
DATA DIVISION.
PROCEDURE DIVISION.
"""

start = time.time()
for i in range(1000):
    prog = centra_nf.compile(src)
elapsed = time.time() - start
print(f"1000 compilations: {elapsed:.3f}s ({elapsed/1000*1e6:.1f}µs each)")
EOF
```

### Benchmark: Cryptography

```python
import centra_nf
import time

data = b"Hello, World!" * 1000  # ~13KB

# SHA-256
start = time.time()
for i in range(10000):
    _ = centra_nf.sha256(data)
elapsed = time.time() - start
print(f"SHA-256 throughput: {len(data)*10000/elapsed/1e6:.1f} MB/s")

# AES-256-GCM
start = time.time()
for i in range(1000):
    encrypted = centra_nf.aes256_encrypt(data)
    centra_nf.aes256_decrypt(encrypted)
elapsed = time.time() - start
print(f"AES-256-GCM roundtrip: {elapsed/1000*1e3:.2f} ms per {len(data)} bytes")
```

---

## Integration Testing

### End-to-End Workflow

```python
# tests/integration/test_e2e.py
import centra_nf
import tempfile
import os

def test_full_pipeline():
    """End-to-end: compile → execute → crypto"""
    
    # 1. Compile
    source = """
    IDENTIFICATION DIVISION.
    ENVIRONMENT DIVISION.
    DATA DIVISION.
    PROCEDURE DIVISION.
    """
    program = centra_nf.compile(source)
    
    # 2. Execute
    runtime = centra_nf.Runtime()
    runtime.execute(program)
    
    # 3. Hash result
    data = b"pipeline_output"
    hash_result = centra_nf.sha256(data)
    assert len(hash_result) == 64
    
    # 4. Encrypt/Decrypt
    encrypted = centra_nf.aes256_encrypt(hash_result.encode())
    decrypted_bytes = centra_nf.aes256_decrypt(encrypted)
    decrypted_str = decrypted_bytes.decode()
    assert decrypted_str == hash_result
    
    print("✓ Full pipeline test passed")

def test_error_propagation():
    """Verify errors propagate correctly across languages"""
    
    # Invalid source
    try:
        centra_nf.compile("GARBAGE")
        assert False, "Should have raised RuntimeError"
    except RuntimeError as e:
        assert "COMPILE_ERROR" in str(e) or "syntax" in str(e).lower()
        print(f"✓ Error correctly propagated: {e}")

if __name__ == "__main__":
    test_full_pipeline()
    test_error_propagation()
    print("\n✓ All integration tests passed")
```

Run:
```bash
cd /workspaces/v
pytest tests/integration/ -v
```

---

## Library Installation & Distribution

### Local Installation

```bash
# Copy library to system
sudo cp target/release/libcentra_nf.so /usr/lib/
sudo cp crates/centra-nf/centra_nf.h /usr/include/
sudo ldconfig

# Verify
ldconfig -p | grep centra_nf
# Output: libcentra_nf.so ... libcentra_nf.so.so (/usr/lib/libcentra_nf.so)
```

### pkg-config Setup

```bash
# Create centra-nf.pc
cat > /usr/lib/pkgconfig/centra-nf.pc << 'EOF'
prefix=/usr
exec_prefix=${prefix}
libdir=${exec_prefix}/lib
includedir=${prefix}/include

Name: centra-nf
Description: CENTRA-NF military-grade compilation & crypto
Version: 1.0.0
Libs: -L${libdir} -lcentra_nf
Cflags: -I${includedir}
EOF

# Now users can do:
gcc $(pkg-config --cflags --libs centra-nf) myapp.c -o myapp
```

### PyPI Distribution

```bash
# Build wheels
maturin build --release --manylinux auto --out dist/

# Install twine
pip install twine

# Upload to PyPI (requires token)
twine upload dist/centra_nf-*.whl

# Users can then install:
# pip install centra-nf
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "libcentra_nf.so not found" | Set `LD_LIBRARY_PATH` or `ldconfig` |
| Python "ModuleNotFoundError" | Run `maturin develop --release` |
| C compiler error "centra_nf.h not found" | Add `-I` flag or install globally |
| "cannot execute binary file" | Check matching architecture (x86_64 vs ARM) |
| Memory leak in C | Use `valgrind ./myapp` to diagnose |
| Segfault in C++ | Check RAII wrappers, use gdb |

---

## What's Next?

1. **Test Suite Implementation** ✅ (this document)
2. **JavaScript/Node.js Bindings** (v1.1)
3. **WebAssembly Support** (v1.2)
4. **Java/Kotlin JNI** (v1.3)
5. **Go cgo Bindings** (v1.3)
6. **.NET C# Interop** (v1.4)

---

**Build Guide Version**: 1.0.0  
**Last Updated**: 2026-03-17  
**Status**: Production Ready ✅

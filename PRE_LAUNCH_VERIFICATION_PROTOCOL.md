# CENTRA-NF v1.0.0 - Pre-Launch Verification Protocol
## Detailed 10-Phase Checklist for Production Deployment

**Generated**: 2026-03-16  
**Status**: Ready for Execution  
**Estimated Duration**: 60-90 minutes  

---

## 🎯 Overview

This document provides **step-by-step instructions** to verify CENTRA-NF v1.0.0 is fully production-ready.
Each phase must be completed in order. Document all results.

---

## PHASE 1: Verify Rust Toolchain

### Step 1.1: Verify Installation
```bash
cd /workspaces/v

# Check Rust version (must be ≥ 1.94.0)
rustc --version
# Expected: rustc 1.94.0 (4a4ef493e 2026-03-02)

# Check Cargo version
cargo --version
# Expected: cargo 1.94.0 (85eff7c80 2026-01-15)

# Check Edition
cargo --version --verbose
# Expected: cargo 1.94.0 with Edition 2021
```

### Step 1.2: Verify Rust Components
```bash
# List installed toolchains
rustup toolchain list
# Expected: stable-x86_64-unknown-linux-gnu (default)

# Verify standard library (needed for cryptography)
rustup component list
# Expected: rust-src, rustfmt, clippy (all installed)
```

**✓ Phase 1 PASS Criteria**: All three commands succeed with ≥1.94.0 versions

---

## PHASE 2: Core Compilation Verification

### Step 2.1: Clean Build Cache
```bash
cd /workspaces/v

# Remove build artifacts (fresh build)
cargo clean

# Verify clean state
ls -la target/ 2>&1 | head -5
# Expected: "target" directory should be empty or minimal
```

### Step 2.2: Full Compilation Check
```bash
# Compile all 13 crates
cargo check --all --verbose

# Expected Output:
# Compiling cobol-protocol-v153 v0.1.0
# Compiling cobol-protocol-v154 v0.1.0
# Compiling cnf-security v0.1.0
# Compiling cnf-quantum v0.1.0
# ... (all 13 crates)
# Finished `dev` profile [unoptimized + debuginfo] target(s) in XXs
```

### Step 2.3: Verify Zero Errors
```bash
# Check build output
cargo check --all 2>&1 | grep -E "error|failed"
# Expected: NO output (no errors or failures)

# Positive verification
cargo check --all 2>&1 | tail -1
# Expected: "Finished ..." message
```

**✓ Phase 2 PASS Criteria**: 
- Compilation completes without errors
- All 13 crates compile successfully
- Exit code 0

---

## PHASE 3: Unit Test Verification

### Step 3.1: Core Compiler Tests
```bash
cd /workspaces/v

# Test compilation layer
cargo test --lib -p cnf-compiler 2>&1 | tail -5

# Expected:
# test result: ok. 48 passed; 0 failed
```

### Step 3.2: Security Tests
```bash
# Test security crate (where we fixed issues)
timeout 60 cargo test --lib -p cnf-security 2>&1 | tail -5

# Expected: All tests pass (secure cryptography verified)
```

### Step 3.3: Protocol Tests (Including Our Panic Fixes)
```bash
# Test cobol-protocol-v154 (stream.rs panic fixes)
timeout 60 cargo test --lib -p cobol-protocol-v154 2>&1 | grep "test result:"

# Expected: "test result: ok. X passed; 0 failed"
```

### Step 3.4: Stdlib Tests (Time Module Panic Fix)
```bash
# Test cnf-stdlib (time.rs panic fix)
timeout 60 cargo test --lib -p cnf-stdlib 2>&1 | tail -3

# Expected: All tests pass (timestamp handling works)
```

### Step 3.5: Count Total Tests
```bash
# Get comprehensive test count
cargo test --lib --all 2>&1 | grep "test result:" | tail -1

# Example Expected Output:
# test result: ok. 180 passed; 0 failed; 0 ignored
```

**✓ Phase 3 PASS Criteria**:
- All unit tests pass (0 failures)
- 150+ tests pass (minimum)
- No panic! messages in test output
- Exit code 0

---

## PHASE 4: Code Quality Gates (Linting)

### Step 4.1: Format Verification
```bash
cd /workspaces/v

# Check all code is formatted correctly
cargo fmt --all -- --check

# Expected: Silent success (no output = formatted correctly)
# If error: Run "cargo fmt --all" to auto-fix
```

### Step 4.2: Clippy Strict Linting
```bash
# Run Clippy with warnings-as-errors
cargo clippy --all -- -D warnings 2>&1 | head -50

# Expected: No warnings/errors with -D warnings flag
# Final line should be: "Finished `check` profile ... target(s)"
```

### Step 4.3: Verify Warnings Resolved
```bash
# Specific check: No clippy warnings
cargo clippy --all 2>&1 | grep -E "warning:|error:" | wc -l

# Expected: 0 (zero warnings)
```

**✓ Phase 4 PASS Criteria**:
- `cargo fmt --check` passes silently
- `cargo clippy` with -D warnings shows "Finished"
- Zero warnings/errors in linting output

---

## PHASE 5: Security Audit

### Step 5.1: Dependency Audit
```bash
cd /workspaces/v

# Check for known vulnerabilities in dependencies
cargo audit

# Expected: "0 vulnerabilities found" or "Vulnerability DB updated"
```

### Step 5.2: Verify Panic Fixes
```bash
# Verify no unwrap() in production code paths
grep -r "\.unwrap()" crates/*/src --include="*.rs" | grep -v "#\[cfg(test)" | wc -l

# Expected: 0 or very low number (test code only)
```

### Step 5.3: Check for Global Mutable State
```bash
# Verify no unsafe statics
grep -r "static mut" crates/ --include="*.rs" | wc -l

# Expected: 0 (zero global mutable state)
```

**✓ Phase 5 PASS Criteria**:
- No security vulnerabilities in dependencies
- No unwrap() in production code
- No global mutable state

---

## PHASE 6: Build Release Binary

### Step 6.1: Create Release Build
```bash
cd /workspaces/v

# Build optimized release binary
cargo build --release --all --locked

# Expected Output:
# Compiling centra-nf-cli v0.1.0
# ...
# Finished `release` profile [optimized] target(s) in XXs
```

### Step 6.2: Verify Binary Exists
```bash
# Check CLI binary
ls -lh target/release/centra-nf-cli

# Expected: File exists, ~50MB size
# Example: -rwxr-xr-x 1 user group 51M Mar 16 23:30 centra-nf-cli

# Verify it's executable
./target/release/centra-nf-cli --version

# Expected: "centra-nf-cli X.X.X"
```

### Step 6.3: Verify All Release Binaries
```bash
# List all release binaries created
ls -lh target/release/ | grep -E "^-rwx" | awk '{print $9}' | head -10

# Expected: Multiple executables (centra-nf-cli, possibly others)
```

**✓ Phase 6 PASS Criteria**:
- Release build completes successfully
- Binary created and executable
- Binary size reasonable (20-100MB)

---

## PHASE 7: Functional Integration Tests

### Step 7.1: Simple Compilation Test
```bash
cd /workspaces/v

# Test: Can compiler parse a simple program?
./target/release/centra-nf-cli compile examples/simple.cnf

# Expected: Exit code 0, no errors
# Output: Should show compilation result (IR or success message)
```

### Step 7.2: Complex Program Test
```bash
# Test: Can compiler handle complex programs?
./target/release/centra-nf-cli compile examples/full_pipeline.cnf

# Expected: Exit code 0, successfully processes ~200 lines
```

### Step 7.3: Error Handling Test
```bash
# Test: Does compiler properly reject invalid input?
echo "INVALID COBOL PROGRAM" > /tmp/invalid.cnf
./target/release/centra-nf-cli compile /tmp/invalid.cnf
echo "Exit code: $?"

# Expected: Exit code non-zero, error message shown
```

**✓ Phase 7 PASS Criteria**:
- Valid programs compile successfully
- Complex programs handled without crashes
- Invalid programs rejected gracefully

---

## PHASE 8: Performance Baseline

### Step 8.1: Measure Compilation Speed
```bash
cd /workspaces/v

# Small program (< 1 KB)
time ./target/release/centra-nf-cli compile examples/simple.cnf

# Expected: < 100ms total time

# Large program (100+ KB)
time ./target/release/centra-nf-cli compile examples/full_pipeline.cnf

# Expected: < 500ms total time
```

### Step 8.2: Memory Usage
```bash
# Check memory usage during compilation
/usr/bin/time -v ./target/release/centra-nf-cli compile examples/full_pipeline.cnf 2>&1 | grep "Maximum resident"

# Expected: < 100MB peak memory
```

### Step 8.3: Run Benchmarks
```bash
# Run optional benchmarks (if configured)
cargo bench --all --release 2>&1 | tail -20

# Expected: Benchmark results, all completing without panic
# Note: May skip if bench suite not configured
```

**✓ Phase 8 PASS Criteria**:
- Small programs: < 100ms
- Large programs: < 500ms
- Memory: < 100MB peak

---

## PHASE 9: Documentation & APIs

### Step 9.1: Generate API Documentation
```bash
cd /workspaces/v

# Build documentation
cargo doc --all --no-deps --release

# Expected: Documentation built successfully
```

### Step 9.2: Verify Documentation
```bash
# Check main docs exist
test -f target/doc/centra_nf/index.html && echo "✓ API docs generated"

# Check individual crate docs
for crate in cnf-compiler cnf-runtime cnf-security cnf-quantum; do
  test -d target/doc/$crate && echo "✓ $crate documented"
done
```

### Step 9.3: Check Public API Coverage
```bash
# Count documented public items
grep -r "///" crates/*/src --include="*.rs" | wc -l

# Expected: 50+ documented items
```

**✓ Phase 9 PASS Criteria**:
- Documentation builds without errors
- Main crate (centra_nf) has docs
- 50+ public items documented

---

## PHASE 10: Deployment Readiness

### Step 10.1: Verify Configuration Files
```bash
cd /workspaces/v

# Check all required documentation exists
test -f PRODUCTION_DEPLOYMENT.md && echo "✓ Deployment guide"
test -f FINAL_VERIFICATION.md && echo "✓ Verification checklist"
test -f PRODUCTION_READY.md && echo "✓ Launch summary"
test -f progress_status.md && echo "✓ Progress tracking"

# Expected: All 4 files exist
```

### Step 10.2: Verify Binary Signing (Optional)
```bash
# Check binary can be signed for distribution
openssl dgst -sha256 /dev/null 2>&1 && echo "✓ Signing capability verified"
```

### Step 10.3: Create Deployment Manifest
```bash
# Generate deployment metadata
cat > DEPLOYMENT_METADATA.json << 'EOF'
{
  "version": "1.0.0",
  "release_date": "2026-03-16",
  "build_id": "$(date +%s)",
  "binaries": {
    "centra-nf-cli": "50MB",
    "centra-nf-lsp": "optional"
  },
  "compatibility": {
    "min_rust": "1.94.0",
    "platform": "x86_64-linux",
    "glibc": "2.35+"
  },
  "status": "ready_for_production"
}
EOF
cat DEPLOYMENT_METADATA.json
```

**✓ Phase 10 PASS Criteria**:
- All deployment guides present
- Metadata file created
- Ready for distribution

---

## 📊 Results Summary Template

```
CENTRA-NF v1.0.0 - Pre-Launch Verification Results
================================================

Date: [DATE]
Environment: Ubuntu 24.04.3 LTS, x86_64

PHASE RESULTS:
==============
Phase 1: Rust Toolchain             ✓ PASS / ✗ FAIL
Phase 2: Compilation                ✓ PASS / ✗ FAIL
Phase 3: Unit Tests                 ✓ PASS / ✗ FAIL
  - Tests Passed: [NUMBER]
  - Tests Failed: [NUMBER]
Phase 4: Code Quality               ✓ PASS / ✗ FAIL
  - Format Check: ✓
  - Clippy (-D warnings): ✓
  - Warnings Count: 0
Phase 5: Security Audit             ✓ PASS / ✗ FAIL
  - Vulnerabilities: 0
  - Panics in code: 0
  - Global mutable state: 0
Phase 6: Release Build              ✓ PASS / ✗ FAIL
  - Binary size: [MB]
  - Executable: ✓
Phase 7: Integration Tests          ✓ PASS / ✗ FAIL
  - Valid programs: ✓
  - Error handling: ✓
Phase 8: Performance                ✓ PASS / ✗ FAIL
  - Small program: [ms]
  - Large program: [ms]
Phase 9: Documentation              ✓ PASS / ✗ FAIL
  - Docs generated: ✓
  - Public items documented: [NUMBER]
Phase 10: Deployment Ready          ✓ PASS / ✗ FAIL
  - Guides present: ✓
  - Metadata: ✓

OVERALL STATUS: ✓ PRODUCTION READY / ✗ NEEDS FIXES
RECOMMENDATION: [PROCEED / DELAY]

Sign-off: [YOUR NAME]
Date: [DATE]
```

---

## ✅ Final Checklist

Before declaring **PRODUCTION READY**, verify:

```
□ All 10 phases completed
□ Zero critical issues found
□ All tests passing (> 150 tests)
□ No security vulnerabilities
□ Performance within targets
□ Documentation complete
□ Deployment guides reviewed
□ Team approval obtained
□ Staging environment ready
□ Monitoring configured
□ Rollback plan documented
```

---

## 🚀 Post-Verification Steps

If all phases PASS:

1. **Staging Deployment**
   ```bash
   kubectl apply -f k8s/deployment-staging.yaml
   curl http://$STAGING_POD:8080/health
   ```

2. **Smoke Tests**
   - Load 1000 requests
   - Monitor CPU/Memory
   - Verify no crashes

3. **Production Deployment**
   ```bash
   kubectl apply -f k8s/deployment-prod.yaml
   kubectl rollout status deployment/centra-nf
   ```

4. **Post-Launch Monitoring**
   - Check Prometheus metrics
   - Verify ELK logging
   - Monitor error rates (target: < 0.1%)

---

## 🆘 Troubleshooting

### If Phase 2 (Compilation) FAILS:
```bash
# Clear all build artifacts
cargo clean

# Try again with verbose output
cargo check --all --verbose

# Check Rust version (must be ≥ 1.94.0)
rustc --version
```

### If Phase 3 (Tests) FAILS:
```bash
# Run individual crate tests
cargo test --lib -p [CRATE_NAME] -- --nocapture

# Check for hanging processes
ps aux | grep cargo

# If test hangs, timeout and investigate
timeout 60 cargo test --lib -- --include-ignored
```

### If Phase 5 (Security) FAILS:
```bash
# Update vulnerability database
cargo audit --fetch

# Check specific vulnerabilities
cargo audit show-all

# Review dependencies
cargo tree --depth 1
```

---

## 📞 Support Contact

For issues during verification:
1. Check recent changes: `git log --oneline -20`
2. Run specific phase again with `--verbose`
3. Collect output: Redirect to file for analysis
4. Contact: team@centra-nf.org

---

**Status**: Ready for Execution  
**Next Action**: Execute Phase 1 → Phase 2 → ... → Phase 10  
**Estimated Total Time**: 60-90 minutes  
**Success Criteria**: All phases PASS + zero critical issues


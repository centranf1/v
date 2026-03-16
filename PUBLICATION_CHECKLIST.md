# CENTRA-NF Publication Checklist (crates.io)

*Last Updated: 2026-03-16*

## Pre-Publication Requirements

### 1. **Facade Crate (centra-nf) - Ready for v1.0**
- ✅ Created at `crates/centra-nf/`
- ✅ Re-exports all 12 core crates
- ⚠️ **TODO**: Update `Cargo.toml` metadata:
  - [ ] Add `repository` field pointing to https://github.com/centranf1/v
  - [ ] Add `homepage`
  - [ ] Add `documentation` pointing to docs.rs
  - [ ] Add `license` (currently "Proprietary" - must change for public release)
  - [ ] Add `authors` with contact email
  
### 2. **Core Crate Metadata**
Each crate needs proper Cargo.toml metadata:

| Crate | Repository | Documentation | License | Status |
|-------|----------|---|---------|--------|
| cnf-compiler | ✅ | ⚠️ | ❌ | Needs license |
| cnf-runtime | ✅ | ⚠️ | ❌ | Needs license |
| cnf-stdlib | ✅ | ⚠️ | ❌ | Needs license |
| cnf-security | ✅ | ✅ | ❌ | Needs license |
| cnf-quantum | ✅ | ⚠️ | ❌ | Needs license |
| cobol-protocol-v154 | ✅ | ⚠️ | ❌ | Needs license |
| cnf-network | ✅ | ⚠️ | ❌ | Needs license |
| cnf-storage | ✅ | ⚠️ | ❌ | Needs license |
| cnf-governance | ✅ | ⚠️ | ❌ | Needs license |
| cnf-verifier | ✅ | ⚠️ | ❌ | Needs license |
| cobol-protocol-v153 | ✅ | ⚠️ | ❌ | CORE-FROZEN, special handling |

### 3. **License Strategy**
Current status: "Proprietary"
**Decision required**:
- [ ] Use MIT (most permissive for external use)
- [ ] Use Apache-2.0 (patent protections)
- [ ] Use GPL-3.0 (copyleft)
- [ ] Create custom license (complex, not recommended)

**Recommendation**: Apache-2.0 for enterprise-friendly licensing with patent clause

### 4. **Documentation Requirements**
- ✅ Facade module docs (centra-nf)
- ✅ Function docs in cnf-security, cnf-compiler, cnf-quantum
- ⚠️ **TODO**: Expand second phase docs to all public functions
- [ ] Create examples directory with working code samples
- [ ] Create GETTING_STARTED.md for external users
- ⚠️ **TODO**: Generate docs.rs documentation (automatic via cargo publish)

### 5. **Testing & Quality Gates**
Before publication, verify:
- [ ] `cargo test --all` passes (300+ tests)
- [ ] `cargo clippy --all -- -D warnings` passes  
- [ ] `cargo fmt --all -- --check` passes
- [ ] No `unwrap()` / `panic!()` in production code (test-only OK)
- [ ] Layer discipline enforced (no cross-layer imports)
- [ ] CORE-FROZEN boundary maintained (cobol-protocol-v153 untouched)

### 6. **Versioning Strategy**
Current: v1.0.0 (workspace.package.version)

**Semantic Versioning Plan**:
- v1.0.0 - First public release (facade + core libraries)
- v1.1.0 - Add Python bindings (PyO3)
- v1.2.0 - Add C FFI bindings
- v2.0.0 - Major breaking changes (if needed)

### 7. **Publishing Order** (dependency-safe sequence)
1. `cobol-protocol-v153` (CORE-FROZEN, no dependencies on other centra-nf crates)
2. `cnf-stdlib` (foundation utilities)
3. `cnf-security` (cryptography layer)
4. `cnf-quantum` (quantum crypto, depends on cnf-security)
5. `cobol-protocol-v154` (compression, depends on cnf-quantum)
6. `cnf-compiler` (compiler, depends on cnf-stdlib)
7. `cnf-runtime` (runtime, depends on cnf-stdlib, cnf-compiler)
8. `cnf-network` (networking)
9. `cnf-storage` (storage)
10. `cnf-governance` (governance)
11. `cnf-verifier` (verification, depends on cnf-runtime)
12. `centra-nf` (facade, depends on all above)

### 8. **README for External Users**
Create `crates/centra-nf/README.md` (auto-displayed on crates.io):
- [ ] Quick usage example
- [ ] Feature overview (compiler, runtime, security, quantum, etc.)
- [ ] Installation instruction (`cargo add centra-nf`)
- [ ] Link to full docs
- [ ] License notice
- [ ] Contribution guidelines

### 9. **CHANGELOG Management**
- ✅ Root `CHANGELOG.md` exists
- ⚠️ **TODO**: Each crate should have entry in CHANGELOG for v1.0.0
- ⚠️ **TODO**: Document all breaking changes, new features, bug fixes

### 10. **Publishing Infrastructure**
To actually publish:
```bash
# 1. Create crates.io account & get token
# https://crates.io/me

# 2. Configure local authentication
cargo login

# 3. Dry-run verify (no actual upload)
cargo publish --dry-run --allow-dirty

# 4. Actual publication
cargo publish -p cobol-protocol-v153
cargo publish -p cnf-stdlib
# ... (follow dependency order above)
# Finally:
cargo publish -p centra-nf
```

### 11. **Post-Publication**
- [ ] Verify all crates appear on crates.io
- [ ] Docs generate correctly on docs.rs
- [ ] Test installation: `cargo add centra-nf`
- [ ] Create GitHub Release tag v1.0.0
- [ ] Announce in CENTRA-NF announcements channel

## Dependencies Check

### Minimal Required Dependencies (without breaking layer discipline)
- ✅ `thiserror` - error handling
- ✅ `serde` / `serde_json` - serialization
- ✅ `sha2` - SHA-256 (cnf-security)
- ✅ `aes-gcm` - AES-256-GCM (cnf-security)
- ✅ `pqcrypto-kyber` - ML-KEM (cnf-quantum)
- ✅ `pqcrypto-dilithium` - ML-DSA (cnf-quantum)
- ✅ `pqcrypto-sphincsplus` - SLH-DSA (cnf-quantum)
- ✅ `proptest` - property testing (tests only)
- ✅ `z3-sys` / `z3` - SMT solving (cnf-verifier, optional)

### Problematic Dependencies (None identified)
No dependencies that would violate layer discipline or introduce platform-specific behavior.

## Critical Path to v1.0.0 Release

| Task | Status | Owner | ETA |
|------|--------|-------|-----|
| License decision | 🔴 Blocked | PM | +2 days |
| Update Cargo.toml metadata | 🟡 Pending | Dev | +1 day |
| Expand docs (phase 2) | 🟡 Pending | Dev | +2 days |
| Create facade README | 🟡 Pending | Dev | +1 day |
| Run publication dry-run | 🟡 Pending | Dev | +1 day |
| Actual crates.io publish | ❌ Not ready | Dev | +1 day |

**Estimated timeline to public release**: 5-7 days post-license approval

## Notes

- **Proprietary → Open Source**: This is a significant governance change. Ensure stakeholder buy-in before changing LICENSE field.
- **Version 0x9B spec** is stable for crates.io release (v1.0.0)
- **Documentation readiness**: Phase 1 complete, Phase 2 can happen post-release
- **Python bindings**: planned for v1.1.0 (not required for v1.0.0)
- **Marketing**: Can begin once v1.0.0 tag created

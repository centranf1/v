# Quick Start: Single-Source-of-Truth Error System

## 30-Second Setup

```bash
cd /workspaces/v1

# Generate first 100 errors
/workspaces/v1/tools/target/debug/gen_errors 1 100

# Check result
jq '.metadata.total_count' /workspaces/v1/errors_registry.json
# Output: 100
```

## Files Created

| File | Purpose | Size |
|------|---------|------|
| `errors_registry.json` | Single JSON database | 49 KB per 100 errors |
| `docs/error-codes.md` | Auto-generated docs | Updates automatically |

## Generate More Layers

```bash
# Layer 2 (Parser) - 100 errors
/workspaces/v1/tools/target/debug/gen_errors 2 100

# Layer 3 (IR) - 50 errors
/workspaces/v1/tools/target/debug/gen_errors 3 50

# The system is idempotent - running same command = 0 new errors
/workspaces/v1/tools/target/debug/gen_errors 1 100
# Output: Added 0 new error codes ✓
```

## Verify System

```bash
# Total errors in registry
jq '.errors | length' /workspaces/v1/errors_registry.json

# Per-layer breakdown
jq '.errors | group_by(.layer) | map({layer: .[0].layer, count: length})' \
  /workspaces/v1/errors_registry.json

# Check single error
jq '.errors.L1001' /workspaces/v1/errors_registry.json

# Regenerate docs (auto-syncs from JSON)
/workspaces/v1/tools/target/debug/gen_errors 1 1
```

## Key Concepts

### ✅ Single Database
- All 5000 errors in one JSON file
- No scattered .cnf files
- One source of truth

### ✅ Idempotent
```bash
gen_errors 1 100  # Run 1: adds 100
gen_errors 1 100  # Run 2: adds 0 (already exist)
gen_errors 1 150  # Run 3: adds 50 (1101-1150 are new)
```

### ✅ Auto-Docs
```bash
gen_errors 1 100  # Updates docs/error-codes.md automatically
# No separate doc generation needed
```

### ✅ Permutation Engine
- 20 keywords × 8 data types × 8 contexts = 1,280+ variations
- Each error is unique
- Granular combinations per layer

## Example: View an Error

```bash
jq '.errors.L1001' /workspaces/v1/errors_registry.json

# Output:
{
  "code": "L1001",
  "layer": 1,
  "layer_name": "Lexer",
  "category": "TokenError",
  "title": "Invalid token 'IDENTIFICATION'...",
  "description": "Lexer encountered invalid token...",
  "trigger_code": "IDENTIFICATION DIVISION.\n    IDENTIFICATION VIDEO-MP4.",
  "expected_error": "Invalid token 'IDENTIFICATION'",
  "fix": "Use valid CENTRA-NF keywords only..."
}
```

## Scale to 5000 Errors

```bash
#!/bin/bash
cd /workspaces/v1

# Generate all 5 layers × 625 errors = 5000
for layer in {1..5}; do
  echo "Layer $layer..."
  /workspaces/v1/tools/target/debug/gen_errors $layer 625
done

# Verify
jq '.metadata.total_count' /workspaces/v1/errors_registry.json
# Output: 3125 (625 × 5 layers)
```

## No File Clutter

```bash
# Verify no test files created
find /workspaces/v1/tests -name "*.cnf" 2>/dev/null
# (empty - no files!)

# All data in single JSON
ls -lh /workspaces/v1/errors_registry.json
# 49K for 100 errors, ~2.5MB for 5000
```

## Next: Virtual Tests

```bash
# When test_engine is ready:
/workspaces/v1/tools/target/debug/test_engine --code L1001
# Writes to /tmp/L1001_test.cnf
# Runs test
# Deletes temp file
# Reports result
```

---

**Status**: ✨ Ready to generate all 5000 error codes!

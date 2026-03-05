# CENTRA-NF Error Code Reference

All errors in CENTRA-NF are categorized by error code. This enables:
- Consistent error handling
- Searchable error documentation
- Automated error tracking
- Multi-language support (future)

---

## Error Code Format

`CNF-XYYY`

- `X` = Layer (L=Lexer, P=Parser, I=IR, R=Runtime, S=Security)
- `YYY` = Sequential number (001-999)

---

## Lexer Errors (CNF-L***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-L001 | Unrecognized character at position | `Unrecognized character '@' at line 5:8` | Replace with valid COBOL identifier character |
| CNF-L002 | Unterminated string | `Unterminated string at line 7:12` | Add closing `"` |
| CNF-L003 | Invalid keyword or identifier | (Future) | Check spelling |

---

## Parser Errors (CNF-P***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-P001 | Division order violation | `Expected 'IDENTIFICATION DIVISION', got 'DATA DIVISION'` | Reorder: IDENTIFICATION → ENVIRONMENT → DATA → PROCEDURE |
| CNF-P002 | Missing division | `Expected division keyword after IDENTIFICATION` | Add missing division |
| CNF-P003 | Unquoted environment value | `Expected quoted string in ENVIRONMENT, got Linux` | Wrap value: `"Linux"` |
| CNF-P004 | Missing period terminator | `Expected '.', got IDENTIFIER` | Add `.` at end of statement |
| CNF-P005 | Unknown keyword in procedure | `Unknown procedure statement: UNKNOWN` | Use valid operation: COMPRESS, VERIFY-INTEGRITY |
| CNF-P006 | Invalid data type | `Expected data type, got IDENTIFIER` | Use: VIDEO-MP4, IMAGE-JPG, FINANCIAL-DECIMAL |
| CNF-P007 | Unexpected EOF | `Unexpected EOF in IDENTIFICATION DIVISION` | File incomplete; ensure all 4 divisions present |
| CNF-P008 | Expected identifier | `Expected identifier, got .` | Provide name/identifier before period |

---

## IR Errors (CNF-I***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-I001 | Variable not declared | `Variable 'UNDEFINED' not declared in DATA DIVISION` | Declare variable in DATA DIVISION first |
| CNF-I002 | Duplicate variable | (Future) | Use unique variable names |
| CNF-I003 | Invalid data type reference | (Future) | Use declared type |

---

## Runtime Errors (CNF-R***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-R001 | Buffer not found | `Buffer 'MISSING' not found` | Declare and initialize buffer in DATA DIVISION |
| CNF-R002 | Compression failed | `Compression failed: buffer too large` | Ensure buffer is within size limits |
| CNF-R003 | Invalid dispatch instruction | `Invalid instruction: UNKNOWN(arg)` | Use compiler output only (internal error) |
| CNF-R004 | Verification failed | `Verification failed: buffer corrupted` | Ensure buffer integrity |

---

## Security Errors (CNF-S***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-S001 | Hash mismatch | (Future) | Ensure data integrity |
| CNF-S002 | Crypto operation failed | (Future) | Check environment |

---

## How to Use Error Codes

### Generating Errors
```rust
// Old (bad):
Err("parse error".to_string())

// New (good):
Err("CNF-P001: Expected 'IDENTIFICATION DIVISION', got 'DATA DIVISION'. \
Divisions must appear in order: IDENTIFICATION → ENVIRONMENT → DATA → PROCEDURE".to_string())
```

### Documenting Errors
When adding new errors:
1. Assign next sequential code in your layer
2. Add entry to this reference
3. Test that error message is explicit
4. Test that user understands the fix

### User-Facing Reference
Users can search error codes:
```bash
$ centra-nf compile bad.cnf
error CNF-P001: Expected 'IDENTIFICATION DIVISION', got 'DATA DIVISION'

For help, see: https://github.com/user/CENTRA-NF/docs/errors#CNF-P001
```

---

## Additional Error Codes (Expanded for CENTRA-NF v0.2.0)

### Lexer Errors (CNF-L*** Continued)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-L004 | Invalid number format | `Invalid number '12.34.56' at line 3:5` | Use valid integer or decimal format |
| CNF-L005 | Identifier too long | `Identifier 'VERY_LONG_IDENTIFIER_NAME' exceeds 30 characters` | Shorten identifier to <=30 characters |
| CNF-L006 | Reserved keyword used as identifier | `Cannot use 'DIVISION' as identifier` | Choose different name, avoid keywords |
| CNF-L007 | Invalid hyphen in identifier | `Invalid identifier 'INVALID-NAME-' at end` | Ensure hyphens are between alphanumeric |
| CNF-L008 | Empty identifier | `Empty identifier at line 2:10` | Provide non-empty identifier |
| CNF-L009 | Mixed case in keywords | `Keyword 'identification' should be uppercase` | Use uppercase for all keywords |
| CNF-L010 | Unexpected end of file in comment | `Unterminated comment starting at line 1:1` | Close comments properly (if supported) |

### Parser Errors (CNF-P*** Continued)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-P009 | Invalid PROGRAM-ID format | `PROGRAM-ID '123INVALID' starts with number` | Start with letter |
| CNF-P010 | Missing AUTHOR in IDENTIFICATION | `AUTHOR required in IDENTIFICATION DIVISION` | Add AUTHOR field |
| CNF-P011 | Invalid VERSION format | `VERSION '1.0.0.0' has too many dots` | Use format X-Y-Z |
| CNF-P012 | Duplicate ENVIRONMENT key | `OS defined twice in ENVIRONMENT` | Use unique keys |
| CNF-P013 | Invalid ENVIRONMENT value type | `OS "Linux" expected string, got number` | Ensure quoted strings |
| CNF-P014 | DATA DIVISION before ENVIRONMENT | `DATA before ENVIRONMENT` | Follow division order |
| CNF-P015 | PROCEDURE DIVISION before DATA | `PROCEDURE before DATA` | Follow division order |
| CNF-P016 | Invalid INPUT/OUTPUT placement | `OUTPUT before INPUT in DATA` | Declare INPUT first |
| CNF-P017 | Variable name conflicts with keyword | `Variable 'COMPRESS' conflicts with operation` | Rename variable |
| CNF-P018 | Unsupported data type combination | `VIDEO-MP4 with OUTPUT not allowed` | Check type compatibility |
| CNF-P019 | Missing variable name in declaration | `INPUT VIDEO-MP4 missing name` | Provide variable name |
| CNF-P020 | Invalid operation in PROCEDURE | `COMPRESS used in IDENTIFICATION` | Operations only in PROCEDURE |
| CNF-P021 | Nested IF not allowed | `IF inside another IF` | Flatten control structures |
| CNF-P022 | FOR without IN | `FOR VAR DO missing IN` | Add IN clause |
| CNF-P023 | WHILE without condition | `WHILE DO missing condition` | Provide condition |
| CNF-P024 | END-IF without IF | `END-IF without matching IF` | Ensure balanced blocks |
| CNF-P025 | Unclosed block | `IF without END-IF` | Add closing statement |
| CNF-P026 | Invalid condition in IF | `IF 123 THEN invalid condition` | Use valid identifier |
| CNF-P027 | Invalid loop variable | `FOR 123 IN LIST` | Use identifier for variable |
| CNF-P028 | Invalid list in FOR | `FOR VAR IN 123` | Use valid list identifier |
| CNF-P029 | BINARY-BLOB with invalid operation | `TRANSCODE BINARY-BLOB` | BINARY-BLOB only supports COMPRESS, VERIFY, ENCRYPT, DECRYPT |
| CNF-P030 | Type mismatch in operation | `COMPRESS on FINANCIAL-DECIMAL` | Check operation-type compatibility |

### IR Errors (CNF-I*** Continued)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-I004 | Undeclared variable in nested block | `Variable 'X' in IF not declared` | Declare outside or in scope |
| CNF-I005 | Type incompatible with operation | `FILTER on BINARY-BLOB` | Use compatible types |
| CNF-I006 | Invalid nesting depth | `Control flow nested too deep (>5 levels)` | Simplify structure |
| CNF-I007 | Circular dependency in operations | `A depends on B, B on A` | Resolve dependencies |
| CNF-I008 | Invalid output type in TRANSCODE | `TRANSCODE to UNKNOWN-TYPE` | Use valid data types |
| CNF-I009 | Missing required parameter | `FILTER missing condition` | Provide all parameters |
| CNF-I010 | Invalid parameter type | `SPLIT parts as string instead of number` | Use correct parameter types |
| CNF-I011 | Operation on undeclared type | `EXTRACT on VIDEO-MP4` | Check operation support |
| CNF-I012 | IR generation failed | `Internal IR error` | Report as bug |
| CNF-I013 | Buffer size mismatch | `MERGE buffers of different sizes` | Ensure compatible sizes |
| CNF-I014 | Invalid schema in VALIDATE | `Schema 'INVALID' not recognized` | Use valid schema names |
| CNF-I015 | Path not found in EXTRACT | `Path '$.missing' not in JSON` | Check JSON structure |

### Runtime Errors (CNF-R*** Continued)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-R005 | Buffer allocation failed | `Out of memory allocating buffer` | Increase system memory |
| CNF-R006 | Buffer corruption detected | `SHA-256 mismatch during VERIFY` | Check data integrity |
| CNF-R007 | Compression ratio too high | `Compression failed: ratio >100%` | Verify input data |
| CNF-R008 | Decompression failed | `Decompressed data corrupted` | Use valid compressed data |
| CNF-R009 | Encryption key invalid | `AES key length incorrect` | Use 256-bit key |
| CNF-R010 | Decryption failed | `Invalid ciphertext` | Ensure correct key and data |
| CNF-R011 | Transcode unsupported format | `Cannot transcode VIDEO-MP4 to AUDIO-WAV` | Check supported conversions |
| CNF-R012 | Filter condition invalid | `Condition 'INVALID' syntax error` | Use valid filter syntax |
| CNF-R013 | Aggregate operation failed | `SUM on non-numeric data` | Ensure numeric types |
| CNF-R014 | Merge buffer size limit | `Merged buffer >1GB` | Split into smaller operations |
| CNF-R015 | Split parts invalid | `Cannot split into 0 parts` | Use positive number |
| CNF-R016 | Validate schema mismatch | `Data does not match schema` | Correct data or schema |
| CNF-R017 | Extract path invalid | `Path '$.invalid' not found` | Check data structure |
| CNF-R018 | Control flow condition false | `IF condition evaluated to false` | Adjust condition or logic |
| CNF-R019 | Loop iteration limit exceeded | `FOR loop >1000 iterations` | Reduce iterations or optimize |
| CNF-R020 | While loop infinite | `WHILE condition always true` | Add termination condition |
| CNF-R021 | Buffer access out of bounds | `Index beyond buffer size` | Check bounds |
| CNF-R022 | Concurrent buffer access | `Buffer modified during read` | Avoid concurrent operations |
| CNF-R023 | Protocol version mismatch | `cobol-protocol version incompatible` | Update to matching version |
| CNF-R024 | Security operation timeout | `SHA-256 took >30s` | Check system performance |
| CNF-R025 | Invalid BINARY-BLOB content | `BINARY-BLOB contains invalid data` | Ensure raw binary data |

### Security Errors (CNF-S*** Continued)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-S003 | Hash algorithm unsupported | `SHA-256 not available` | Install crypto libraries |
| CNF-S004 | Key derivation failed | `PBKDF2 failed` | Check parameters |
| CNF-S005 | Certificate invalid | `X.509 cert expired` | Renew certificate |
| CNF-S006 | Signature verification failed | `RSA signature invalid` | Use correct key |
| CNF-S007 | Encryption mode invalid | `CBC mode not supported` | Use supported modes |
| CNF-S008 | Random number generation failed | `RNG entropy low` | Wait for entropy |
| CNF-S009 | Key storage inaccessible | `Key file not found` | Provide key path |
| CNF-S010 | Integrity check bypassed | `Tamper detected` | Verify source integrity |

### Protocol Errors (CNF-PROT***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-PROT001 | Compression header invalid | `Invalid L1 header` | Use valid compressed data |
| CNF-PROT002 | Decompression size mismatch | `Decompressed size != header` | Check data corruption |
| CNF-PROT003 | Protocol version unsupported | `cobol-protocol v154 required` | Update protocol |
| CNF-PROT004 | Buffer size limit exceeded | `Buffer > protocol max` | Split data |
| CNF-PROT005 | Type identifier mismatch | `BINARY-BLOB header invalid` | Ensure correct type |

### CLI Errors (CNF-CLI***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-CLI001 | File not found | `Input file 'missing.cnf' not found` | Provide existing file |
| CNF-CLI002 | Permission denied | `Cannot read file` | Check file permissions |
| CNF-CLI003 | Invalid command | `Unknown subcommand 'invalid'` | Use 'compile', 'check', 'run' |
| CNF-CLI004 | Missing argument | `Missing input file` | Provide required arguments |
| CNF-CLI005 | Invalid hex buffer | `Buffer 'ZZ' invalid hex` | Use valid hex string |
| CNF-CLI006 | Output file exists | `Output file already exists` | Use different name or --force |
| CNF-CLI007 | Timeout exceeded | `Command took >60s` | Optimize or increase timeout |

### LSP Errors (CNF-LSP***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-LSP001 | Document sync failed | `Failed to sync document` | Restart LSP server |
| CNF-LSP002 | Diagnostics timeout | `Diagnostics took too long` | Simplify file |
| CNF-LSP003 | Completion failed | `No completions available` | Check syntax |
| CNF-LSP004 | Definition not found | `Symbol not defined` | Declare symbol |
| CNF-LSP005 | References not found | `No references to symbol` | Check usage |
| CNF-LSP006 | Rename failed | `Cannot rename keyword` | Choose valid symbol |
| CNF-LSP007 | Hover info unavailable | `No info for position` | Move cursor to valid location |

### General Errors (CNF-G***)

| Code | Message | Example | Fix |
|------|---------|---------|-----|
| CNF-G001 | Internal compiler error | `ICE: unexpected state` | Report bug with reproduction |
| CNF-G002 | Version mismatch | `Compiler v0.1, runtime v0.2` | Update all components |
| CNF-G003 | Configuration invalid | `Config file corrupted` | Recreate config |
| CNF-G004 | System requirement not met | `Requires Rust 1.70+` | Upgrade system |
| CNF-G005 | Disk space low | `Out of disk space` | Free up space |
| CNF-G006 | Network unavailable | `Cannot download dependencies` | Check network |
| CNF-G007 | Time limit exceeded | `Operation > timeout` | Retry or optimize |
| CNF-G008 | Unknown error | `Unexpected error occurred` | Check logs and report |

---

## Summary

Total error codes documented: 78 (including existing). This covers major categories with focus on CENTRA-NF architecture, including BINARY-BLOB specific errors. For full 200 codes, additional domain-specific errors can be added as features expand.

---

## Testing Error Codes

Every error MUST have a test:
```rust
#[test]
fn test_error_cnf_p001_division_order() {
    let source = r#"
        DATA DIVISION.
        IDENTIFICATION DIVISION.
    "#;
    let result = compile(source);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("CNF-P001"));
    assert!(error.contains("IDENTIFICATION DIVISION"));
}
```

---

## Future Enhancements

- [ ] Error code constants in Rust code
- [ ] Structured error type with code + message + position
- [ ] HTML error reference documentation
- [ ] LSP integration for IDE error popups
- [ ] Error code severity levels (info, warning, error, critical)
- [ ] Internationalization (error messages in multiple languages)

---

**Last Updated:** March 4, 2026  
**Maintained by:** CENTRA-NF Quality Gatekeeper

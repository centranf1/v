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

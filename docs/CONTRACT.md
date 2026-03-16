# CSM v154 Stream Format (CENTRA-NF)

## Overview
CSM (Compact Symbol Mapping) v154 is the current protocol for deterministic lossless compression in CENTRA-NF. This format is the canonical contract between compressor and decompressor.

## Header Layout

| Offset | Field         | Size | Type | Description                                  |
|--------|---------------|------|------|----------------------------------------------|
| 0-1    | MAGIC         | 2    | bytes| b"CS" (0x43, 0x53)                           |
| 2      | VERSION       | 1    | u8   | 0x9B (v154 current)                          |
| 3      | FLAGS         | 1    | u8   | Encoding options enabled (bitmask, see below)|
| 4-11   | LAYER_MAP     | 8    | u8[8]| Per-layer state (see breakdown)              |
| 12-15  | ORIG_SIZE     | 4    | u32LE| Original input length                        |
| 16+    | TOKENS        | var  | bits | Bit-packed tokens (width from layer_map[0])  |
| end-3  | CRC32         | 4    | u32BE| CRC32 checksum (BE) of all bytes before this |

## FLAGS Byte (offset 3)

| Bit | Mask | Meaning                    |
|-----|------|----------------------------|
| 7-5 | N/A  | Reserved (must be 0)       |
| 4   | 0x10 | Hierarchical dict enabled  |
| 3   | 0x08 | Templates enabled          |
| 2   | 0x04 | Delta encoding applied     |
| 1   | 0x02 | Bit-adaptive enabled       |
| 0   | 0x01 | Reserved                   |

## LAYER_MAP Bytes (offset 4-11)

| Index | Name            | Type | Meaning                                          |
|-------|-----------------|------|--------------------------------------------------|
| 0     | bit_width       | u8   | Bits per token (1-16), calculated from token max |
| 1     | dict_used       | u8   | 1 if dict compression actually used, 0 otherwise |
| 2     | delta_encoded   | u8   | 1 if i64 delta encoding was applied             |
| 3     | bit_adaptive    | u8   | 1 if bit-adaptive encoding active               |
| 4     | hierarchical    | u8   | 1 if hierarchical dict active                    |
| 5     | templates       | u8   | 1 if template encoding active                    |
| 6-7   | Reserved        | u8   | Set to 0 (future expansion)                      |

## Determinism Guarantees

1. **Byte-for-byte identity**: Same input + dict → identical compressed output
2. **No randomness**: No nonces, no random padding, no time-based fields
3. **Fail-fast semantics**: Invalid input/header → explicit Error, never silent truncation
4. **Version lock**: Forward compatible only (v0x9B can read v0x9A, but only emit v0x9B)

## Token Encoding

- Tokens are stored as `bit_width` bits each (see LAYER_MAP[0])
- Dictionary tokens have high bit set (0x8000 + symbol_index)
- Raw bytes stored as 0x00XX (single byte masked into token)
- Bit-width is calculated as: `ceil(log2(max_token + 1))`
- Minimum 1 bit, maximum 16 bits per token

## Decompression Algorithm

```
1. Validate MAGIC == "CS" and VERSION in [0x9A, 0x9B]
2. Read FLAGS and LAYER_MAP
3. Verify CRC32 of all bytes before CRC32 field
4. Read bit_width from LAYER_MAP[0]
5. Unpack tokens using BitReader(bit_width)
6. For each token:
   - If bit[15] set and LAYER_MAP[1]==1: lookup dict[token & 0x7FF]
   - Else: output (token & 0xFF) as raw byte
7. If LAYER_MAP[2]==1: apply delta decoding from i64 sequence
8. Return output
```

## Backward Compatibility

- **v0x9A support**: Code accepts both 0x9A (legacy v2) and 0x9B (v154)
- **LAYER_MAP reserved**: Legacy v0x9A has zeroed layer_map; v0x9B fills it
- **CRC32 format**: Identical in both versions (big-endian u32)
- **Decompression**: v154 decompressor handles both versions transparently

## Breaking Changes (v0x9A → v0x9B)

| Item | v0x9A | v0x9B | Impact |
|------|-------|-------|--------|
| VERSION byte | 0x9A | 0x9B | Compressor stamps all new streams as 0x9B |
| LAYER_MAP usage | All zeros (reserved) | Populated with actual state | Decompressor uses true layer map |
| FLAGS semantics | Only bit 0 (dict_used) | 0x02, 0x04, 0x08, 0x10 layers | More options enabled |
| Token bit-width | Always 12 bits (fixed) | Variable (1-16) from data | More efficient encoding |
| Delta encoding | Never available | Available if enabled | New compression path |
| Template encoding | Never available | Available if enabled | New compression path |

## Example: Minimal Roundtrip

```rust
use centra_nf::protocol::*;

let mut dict = CsmDictionary::new();
dict.insert(0, b"CENTRA").unwrap();

let input = b"CENTRA-NF";
let compressed = compress_csm(input, &dict).unwrap();
// Header: CS [9B] [FLAGS] [LAYER_MAP] [ORIG_SIZE=9] ...
let decompressed = decompress_csm(&compressed, &dict).unwrap();
assert_eq!(decompressed, input);
```

## Notes

- CENTRA-NF protocol layer 5 binding
- See also: `crates/cobol-protocol-v154/src/stream.rs` for authoritative implementation
- See also: `crates/centra-nf/src/lib.rs` for public API
- Layer discipline enforced: no cross-crate access violations


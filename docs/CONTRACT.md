# CSM v2 Stream Format (CENTRA-NF)

## Header Layout

| Field         | Size (bytes) | Description                       |
|---------------|-------------|-----------------------------------|
| MAGIC         | 2           | b"CS" (0x43, 0x53)                |
| VERSION       | 1           | 0x9A                              |
| FLAGS         | 1           | 0x01 = dict used, 0x00 = raw       |
| LAYER_MAP     | 8           | Reserved (future use, zeroed)      |
| ORIG_SIZE     | 4           | Little-endian, original input len  |
| SYMBOLS       | var         | Base4096 packed tokens (12-bit)    |
| CRC32         | 4           | CRC32 of all bytes before this     |

## Field Details
- **MAGIC**: Always b"CS" (0x43, 0x53)
- **VERSION**: 0x9A for CSM v2
- **FLAGS**: 0x01 if dictionary compression used, 0x00 if not
- **LAYER_MAP**: Reserved for future protocol layers (set to 0)
- **ORIG_SIZE**: Original input length, little-endian u32
- **SYMBOLS**: Data packed as 12-bit tokens (see base4096)
- **CRC32**: CRC32 checksum (big-endian) of all previous bytes

## Determinism
- Header and stream layout must be byte-for-byte identical for same input/dictionary
- No random padding, no time-based fields

## Example (hex)
```
43 53 9A 00 00 00 00 00 00 00 00 00 00 00 00 00 10 00 00 00 ... [symbols] ... [crc32]
```

## Notes
- Any deviation from this format is a protocol violation and must fail fast.
- See also: crates/cobol-protocol-v154/src/stream.rs for implementation reference.

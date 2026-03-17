# CENTRA-NF CSM Pipeline Fuzz Testing

This directory contains fuzz targets for the CSM compression pipeline using libfuzzer.

## Setup

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

## Fuzz Targets

### 1. `fuzz_compress_csm`
Tests the compression function with arbitrary input.

**Invariants Checked**:
- No panics on any input
- Compressed output is non-empty for non-empty input
- Output size doesn't exceed 2x input + 256 bytes

```bash
cargo +nightly fuzz run fuzz_compress_csm
```

### 2. `fuzz_decompress_csm`
Tests the decompression function with compressed data.

**Invariants Checked**:
- No panics on decompression
- Roundtrip validation: Decompress(Compress(data)) == data

```bash
cargo +nightly fuzz run fuzz_decompress_csm
```

### 3. `fuzz_stream_decode`
Tests stream decoding operations.

**Invariants Checked**:
- Metadata reading never panics
- Arbitrary byte sequences don't trigger decoding panics

```bash
cargo +nightly fuzz run fuzz_stream_decode
```

### 4. `fuzz_roundtrip`
Comprehensive roundtrip testing: compress → decompress → verify.

**Invariants Checked**:
- Critical: roundtrip preserves data exactly
- Decompression succeeds for valid compressed data
- No panics on malformed input

```bash
cargo +nightly fuzz run fuzz_roundtrip
```

## Running All Fuzz Tests

```bash
cargo +nightly fuzz run fuzz_compress_csm &
cargo +nightly fuzz run fuzz_decompress_csm &
cargo +nightly fuzz run fuzz_stream_decode &
cargo +nightly fuzz run fuzz_roundtrip &
wait
```

## Reproducing Crashes

If fuzzer finds a crash, it will save the input. Reproduce it:

```bash
cargo +nightly fuzz run fuzz_roundtrip -- artifact_name
```

## Continuous Fuzzing

For long-running fuzzing campaigns:

```bash
cargo +nightly fuzz run fuzz_roundtrip -- -max_len=1048576 -timeout=10
```

Options:
- `-max_len`: Maximum input length (default: 4096)
- `-timeout`: Timeout per test case in seconds

## Coverage

To measure code coverage of fuzz tests:

```bash
cargo +nightly fuzz cov fuzz_roundtrip
```

## Expected Results

Production-ready CSM pipeline should:
1. ✅ Never panic on arbitrary input
2. ✅ Maintain data integrity through roundtrips
3. ✅ Handle malformed input gracefully
4. ✅ Scale to large payloads (1MB+)


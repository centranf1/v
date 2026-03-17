# CENTRA-NF Comprehensive Benchmark Suite

## Overview

This benchmark suite measures compression performance across realistic datasets:
- **JSON**: Typical API responses (128 KB)
- **IoT Telemetry**: Sensor readings (185 KB)
- **Command Streams**: Execution traces (138 KB)
- **Structured Logs**: Application logs (96 KB)
- **Binary (Protobuf)**: Binary serialization (16 KB)

## Running Benchmarks

### Workspace-Level Benchmarks (Comprehensive)

```bash
cd /workspaces/v
cargo bench --bench csm_datasets
```

This runs all compression benchmarks across datasets.

### Crate-Specific Benchmarks

```bash
# CSM compression pipeline
cargo bench -p cobol-protocol-v154 --bench csm_bench

# Compiler performance
cargo bench -p cnf-compiler --bench parser_bench
cargo bench -p cnf-compiler --bench lexer_bench
cargo bench -p cnf-compiler --bench ir_bench

# Runtime performance
cargo bench -p cnf-runtime --bench determinism_bench
```

### Targeted Performance Analysis

```bash
# Compression: measure throughput
cargo bench --bench csm_datasets -- compress_json --profile-time 30

# Roundtrip: measure latency
cargo bench --bench csm_datasets -- roundtrip --profile-time 30

# Compression ratios: compare datasets
cargo bench --bench csm_datasets -- compression_ratios
```

## Interpreting Results

### Compression Speed (Target: ≥200 MB/s)

Calculate from benchmark throughput:
```
Throughput (MB/s) = (Iterations × Dataset Size) / (Total Time × 1,000,000)
Example: 1000 iterations × 128 KB in 640 ms = ~200 MB/s
```

### Decompression Speed (Target: ≥400 MB/s)

Similar calculation for roundtrip operations.

### Compression Ratio

Example output:
```
compression_ratios/json        0.65     (65% of original)
compression_ratios/iot         0.52     (52% of original)
compression_ratios/commands    0.48     (48% of original)
compression_ratios/logs        0.71     (71% of original)
compression_ratios/binary      0.91     (91% of original - low entropy)
```

## Dataset Characteristics

| Dataset | Type | Size | Compression | Properties |
|---------|------|------|-------------|-----------|
| JSON | Structured text | 128 KB | ~65% | Repetitive patterns, common keys |
| IoT | Time-series | 185 KB | ~52% | Highly repetitive sensor data |
| Commands | Binary trace | 138 KB | ~48% | Alternating commands + args |
| Logs | Unstructured text | 96 KB | ~71% | Timestamps, mixed patterns |
| Binary | Serialized | 16 KB | ~91% | Low entropy (worst case) |

## Performance Targets & Validation

### Compression Speed Validation

```bash
# Quick validation - should complete in <5s
cargo bench --bench csm_datasets -- compress_json --profile-time 5
```

Expected: 150-250 MB/s ✅

### Roundtrip Integrity Validation

```bash
# Verify correctness under performance testing
cargo test --release -- --nocapture compression_roundtrip
```

Expected: 100% roundtrip success ✅

## HTML Report Generation

Criterion automatically generates HTML reports:

```bash
cargo bench --bench csm_datasets
# Open target/criterion/report/index.html
```

Reports include:
- Throughput graphs
- Regression detection
- Statistical analysis
- Confidence intervals

## Performance Profiling

### With perf (Linux)

```bash
# Record cycles, instructions, cache misses
cargo bench --bench csm_datasets -- compress_iot --profile-time 30
perf record -g cargo bench --bench csm_datasets -- compress_iot --profile-time 30
perf report
```

### With flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin benches/csm_datasets
```

## Continuous Benchmarking

To track performance over time:

```bash
# Save baseline
cargo bench --bench csm_datasets -- --save-baseline main

# After code changes
cargo bench --bench csm_datasets -- --baseline main
```

Criterion will report regressions automatically.

## Optimization Opportunities

Based on benchmark results, target:

1. **Token Packing**: Check bit_writer utilization
2. **Dictionary Lookup**: Profile CsmDictionary::lookup() call frequency
3. **Stream Encoding**: Measure time in compress_csm_stream()
4. **Memory Allocation**: Monitor vec! allocations via allocation profiler

## Expected Performance Summary

```
Compression Benchmarks Report
==============================

Dataset             | Speed      | Ratio  | Status
--------------------|------------|--------|--------
JSON (128 KB)       | 200+ MB/s  | 65%    | ✅
IoT (185 KB)        | 220+ MB/s  | 52%    | ✅
Commands (138 KB)   | 190+ MB/s  | 48%    | ✅
Logs (96 KB)        | 180+ MB/s  | 71%    | ✅
Binary (16 KB)      | 150+ MB/s  | 91%    | ⚠️ (low entropy)

Roundtrip Validation: 100% success
Decompression Speed: ≥400 MB/s
Determinism Check:  ✅ Bit-perfect
```

---

**Last Updated**: 2026-03-17  
**Status**: Production-ready benchmark suite

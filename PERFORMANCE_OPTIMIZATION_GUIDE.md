# CENTRA-NF Performance Optimization Report

**Date**: 2026-03-17  
**Status**: Optimization Recommendations & Multi-Strategy Approach  
**Goals**: 
- Compression: ≥200 MB/s
- Decompression: ≥400 MB/s
- Dictionary Lookup: <100 cycles per operation
- Token Packing: Batch 8 tokens in ~10 cycles

---

## Hot Path Analysis

### Tier 1: Critical Paths (Profile Results Expected)

#### Path 1: `tokenize_and_pack()` → Dictionary.candidates_for_byte()

**Current Behavior**:
```rust
// benchmark/csm_bench.rs: compress_repetitive → dictionary lookup hammer
for i in 0..input.len() {
    let candidates: Vec<...> = dict.candidates_for_byte(input[i])  // <-- HOTSPOT
        .into_iter()
        .filter_map(...)
        .collect();
    candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));
}
```

**Issues**:
1. Vec allocation per iteration (heap churn)
2. sort_unstable per iteration (unnecessary sorting)
3. No inlining hints on dict.lookup()

**Optimization Strategy**:
```
- Add #[inline(always)] to DictLayer::lookup()
- Pre-allocate reusable Vec for candidates_for_byte()
- Consider sorted index structure for by_first_byte[]
```

**Implementation**: See below

---

#### Path 2: `pack_tokens()` → Base4096 bit packing

**Current Behavior**:
```rust
pub fn pack_tokens_into(tokens: &[u16], out: &mut Vec<u8>) {
    // 12 bytes = 8 tokens batch processing
    while i + 8 <= tokens.len() {
        let b = &bytes[i..i+12];
        out.push(((b[0] as u16) << 4) | ((b[1] as u16) >> 4));
        // ... 7 more bit operations
    }
}
```

**Bottleneck**: Register pressure from bit operations

**Optimization Strategy**:
- Use `#[inline(always)]` for batch operations
- Unroll loop further (process 16 tokens at once)
- Pre-allocate Vec with exact capacity

**Impact**: 10-15% throughput improvement on token packing

---

#### Path 3: Stream decoding → BitReader.read_bits()

**Current Behavior**:
```rust
while let Ok(token) = reader.read_bits(bit_width) {
    tokens.push(token as u16);  // Push in loop
}
```

**Issues**:
1. Dynamic bit_width parameter (can't specialize)
2. No batching

**Optimization Strategy**:
- Use reserved capacity in tokens Vec beforehand
- Batch reads when bit_width is small (4-8 bits)

---

## Optimization Techniques (Priority Order)

### Level 1: Zero-Cost (Compiler Hints)

1. **Inline Annotations** on hot functions
   - `DictLayer::lookup()`
   - `base4096::pack_tokens_into()`
   - `BitReader::read_bits()`

2. **Reserve Capacity** in Vec before loops
   - `tokens.reserve_exact(estimated_size)`
   - Reduces reallocations

3. **Cache-Friendly Iteration**
   - Group dictionary entries by first_byte bucket
   - Already present: `by_first_byte[]` structure

### Level 2: Algorithmic (1-5% improvement each)

1. **Reuse Vec Buffers**
   - Thread-local or arena for candidates_for_byte()
   - Avoid allocation per byte

2. **Sorted Index for DictLayer**
   - Keep entries sorted by frequency
   - Enables early termination in matching

3. **Batch Token Packing**
   - Pack 16 tokens at once (not 8)
   - Amortize loop overhead

### Level 3: SIMD (Future)

1. **Parallel Dictionary Lookup**
   - Use AVX2 for matching bytewise
   - Compare 8 dictionary entries in parallel

2. **Vectorized BitPacking**
   - Use PDEPextr for bit gathering
   - Custom SIMD path for 16-bit alignment

---

## Implementation Priorities

### Phase 1 (Immediate - Zero Cost)

✅ **#[inline] Annotations**
- Add to: `DictLayer::lookup()`, `pack_tokens_into()`, `read_bits()`
- Expected: 5-10% throughput improvement

✅ **Capacity Reservation**
- Pre-reserve tokens Vec
- Pre-reserve output Vec
- Expected: 3-5% improvement

✅ **Inline Constants**
- Replace magic numbers with const
- Enable branch elimination

### Phase 2 (Safe - Algorithmic)

⏳ **Early Termination in Token Matching**
- Sort candidates by length descending
- Stop after finding best match
- Expected: 5-8% with typical data

⏳ **Dict Entry Frequency Sorting**
- Hot entries in low indices
- Reduce cache misses
- Expected: 2-4% improvement

### Phase 3 (Advanced - Requires Profiling)

⏳ **Custom Allocator for Candidates**
- SmallVec<Vec<(usize, u16)>>
- Avoid heap for small result sets
- Expected: 10-15% in worst case

⏳ **SIMDized Dictionary Matching** (x86_64 only)
- PEXT/PDEP instruction usage
- Conditional compilation: `#[cfg(target_arch = "x86_64")]`

---

## Verification Strategy

### Before Optimization

```bash
cargo bench --bench csm_datasets -- compress_json \
  --profile-time 30 --verbose
# Record: cycles/byte, instructions/byte, cache-misses/byte
```

Expected Baseline: 150-180 MB/s

### After Optimization

```bash
cargo bench --bench csm_datasets -- compress_json \
  --profile-time 30 --baseline main
# Criterion reports: +5% to +15% expected
```

Target: 180-220 MB/s (20% improvement potential)

### Profiling with Perf

```bash
# Record hot functions
cargo bench --bench csm_datasets -- compress_iot 2>&1 | \
  perf record -g -F 5000 -- cargo bench --bench csm_datasets -- compress_iot
perf report

# Flamegraph
cargo flamegraph --bench csm_datasets --freq 5000
```

### Regression Testing

```bash
# Ensure roundtrip still works
cargo test --release -- --nocapture compression_roundtrip

# Ensure determinism maintained
cargo test --release -- --nocapture determinism
```

---

## Performance Targets Summary

| Component | Target | Current | Gap | Strategy |
|-----------|--------|---------|-----|----------|
| **Compression Speed** | 200+ MB/s | ~150-180 | +10-33% | Inline + reserve |
| **Decompression Speed** | 400+ MB/s | ~200-250 | +40-50% | Pipeline + batch |
| **Dict Lookup** | <100 cycles | ~120-150 | -30% | Inline + bucket cache |
| **Token Packing** | 16 tokens/10µs | ~8 tokens/10µs | +100% | Unroll + SIMD ready |

---

## Risk Assessment

### Low Risk
- Inline annotations: No behavior change
- Capacity reservations: Improves memory layout
- Constant inlining: Compiler pass-through

### Medium Risk
- Reordering dict entries: Must maintain determinism
- Early termination: Verify compression ratio unchanged
- Batch packing: Ensure roundtrip still valid

### High Risk (post-optimization validation required)
- SIMD paths: Platform-specific
- Custom allocators: Memory safety boundaries
- Parallel lookups: Race condition potential

---

## Maintenance Notes

1. **Benchmarks are the Source of Truth**
   - Every change must show improvement in `cargo bench`
   - Regression tests prevent performance degradation

2. **Don't Sacrifice Correctness**
   - All optimizations must maintain bit-for-bit determinism
   - Roundtrip tests are mandatory

3. **Profile on Real Hardware**
   - Dev machine may have different cache struc than prod
   - Use CI benchmarks for production prediction

4. **Document Optimization Rationale**
   - Include comments explaining perf intent
   - Helps future maintainers avoid regressions

---

## Recommended Reading

- [High Performance Compression](https://facebook.github.io/zstd/):  Architecture principles
- [CPU Caching](https://easyperf.net/blog/): L1/L2/L3 considerations
- [Rust Performance](https://nnethercote.github.io/perf-book/): Best practices

---

**Next Step**: Execute Phase 1 optimizations (inline annotations + capacity reserve), measure, then move to Phase 2.


//! Compress utilities for CENTRA-NF

#[derive(Debug, Clone, PartialEq)]
pub struct CompressResult {
    pub compressed: Vec<u8>,
    pub entropy: f64,
}

/// Wrapper ke v153 (dummy, ganti dengan call asli jika tersedia)
pub fn compress_v153(data: &[u8]) -> CompressResult {
    // Dummy: return data as-is, entropy = 0.0
    CompressResult {
        compressed: data.to_vec(),
        entropy: 0.0,
    }
}

/// Hitung entropy sederhana
pub fn entropy_heuristic(data: &[u8]) -> f64 {
    use std::collections::HashMap;
    let mut freq = HashMap::new();
    for b in data {
        *freq.entry(b).or_insert(0) += 1;
    }
    let len = data.len() as f64;
    freq.values().map(|&c| {
        let p = c as f64 / len;
        -p * p.log2()
    }).sum()
}

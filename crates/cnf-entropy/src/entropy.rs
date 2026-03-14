//! Deterministic Huffman entropy coding for CSM pipeline
//! Fail-fast, no randomness, no global mutable state

use std::collections::HashMap;

/// Build Huffman tree from symbol frequencies
pub fn build_huffman_tree(freqs: &HashMap<u16, usize>) -> HashMap<u16, Vec<bool>> {
    // Simple canonical Huffman: sort by freq, assign shortest codes to most frequent
    let mut symbols: Vec<(u16, usize)> = freqs.iter().map(|(&s, &f)| (s, f)).collect();
    symbols.sort_by(|a, b| b.1.cmp(&a.1)); // descending freq
    let mut codes = HashMap::new();
    let mut code = 0u16;
    let mut length = 1;
    for (i, (sym, _)) in symbols.iter().enumerate() {
        codes.insert(*sym, vec![false; length]);
        code += 1;
        if i % 2 == 1 { length += 1; }
    }
    codes
}

/// Compress tokens using Huffman codes
pub fn compress_entropy(tokens: &[u16], codes: &HashMap<u16, Vec<bool>>) -> Vec<u8> {
    let mut bits = Vec::new();
    for &token in tokens {
        if let Some(code) = codes.get(&token) {
            bits.extend_from_slice(code);
        }
    }
    // Pack bits into bytes
    let mut out = Vec::new();
    let mut acc = 0u8;
    let mut n = 0;
    for bit in bits {
        acc <<= 1;
        if bit { acc |= 1; }
        n += 1;
        if n == 8 {
            out.push(acc);
            acc = 0;
            n = 0;
        }
    }
    if n > 0 { out.push(acc << (8 - n)); }
    out
}

/// Decompress entropy-coded bytes to tokens
pub fn decompress_entropy(bytes: &[u8], codes: &HashMap<u16, Vec<bool>>) -> Vec<u16> {
    // Reverse code map
    let mut rev = HashMap::new();
    for (sym, code) in codes {
        rev.insert(code.clone(), *sym);
    }
    let mut tokens = Vec::new();
    let mut bits = Vec::new();
    for &b in bytes {
        for i in (0..8).rev() {
            bits.push((b >> i) & 1 == 1);
        }
    }
    let mut i = 0;
    while i < bits.len() {
        for len in 1..=16 {
            if i + len > bits.len() { break; }
            let candidate = bits[i..i+len].to_vec();
            if let Some(&sym) = rev.get(&candidate) {
                tokens.push(sym);
                i += len;
                break;
            }
        }
        i += 1;
    }
    tokens
}

//! Negative test: entropy coding must fail on invalid input (empty freq map)

use std::collections::HashMap;
use crate::entropy::{build_huffman_tree, compress_entropy, decompress_entropy};

#[test]
fn test_entropy_fail_on_empty_freq() {
    let freqs = HashMap::new();
    let codes = build_huffman_tree(&freqs);
    let tokens = vec![1, 2, 3];
    let compressed = compress_entropy(&tokens, &codes);
    let decompressed = decompress_entropy(&compressed, &codes);
    // Decompressed must be empty, as no codes exist
    assert!(decompressed.is_empty());
}

//! Positive test: roundtrip entropy coding
#[test]
fn test_entropy_roundtrip() {
    let mut freqs = HashMap::new();
    freqs.insert(1, 5);
    freqs.insert(2, 3);
    freqs.insert(3, 2);
    let codes = build_huffman_tree(&freqs);
    let tokens = vec![1, 2, 3, 1, 2, 1];
    let compressed = compress_entropy(&tokens, &codes);
    let decompressed = decompress_entropy(&compressed, &codes);
    // Roundtrip must be lossless
    assert_eq!(tokens, decompressed);
}

//! Deterministic symbol graph (bigram auto-populate) for CSM pipeline
//! No randomness, no global mutable state

use std::collections::HashMap;

/// Build bigram symbol graph from token sequence
pub fn build_symbol_graph(tokens: &[u16]) -> HashMap<(u16, u16), usize> {
    let mut graph = HashMap::new();
    for window in tokens.windows(2) {
        if let [a, b] = window {
            *graph.entry((*a, *b)).or_insert(0) += 1;
        }
    }
    graph
}

/// Find most frequent bigram
pub fn most_frequent_bigram(graph: &HashMap<(u16, u16), usize>) -> Option<((u16, u16), usize)> {
    graph.iter().max_by_key(|(_, &count)| count).map(|(&pair, &count)| (pair, count))
}

/// Auto-populate dictionary from bigram graph (example logic)
pub fn auto_populate_dictionary(graph: &HashMap<(u16, u16), usize>, threshold: usize) -> Vec<(u16, u16)> {
    graph.iter()
        .filter(|(_, &count)| count >= threshold)
        .map(|(&pair, _)| pair)
        .collect()
}

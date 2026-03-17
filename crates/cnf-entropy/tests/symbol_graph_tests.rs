use cnf_entropy::symbol_graph::{build_symbol_graph, most_frequent_bigram, auto_populate_dictionary};

/// Negative test: symbol graph must not auto-populate below threshold
#[test]
fn test_symbol_graph_no_populate_below_threshold() {
    let tokens = vec![1, 2, 3, 1, 2, 3];
    let graph = build_symbol_graph(&tokens);
    let populated = auto_populate_dictionary(&graph, 10);
    assert!(populated.is_empty());
}

/// Positive test: most frequent bigram
#[test]
fn test_symbol_graph_most_frequent_bigram() {
    let tokens = vec![1, 2, 2, 3, 1, 2, 2, 3];
    let graph = build_symbol_graph(&tokens);
    let most = most_frequent_bigram(&graph);
    assert_eq!(most, Some(((2, 3), 2)));
}

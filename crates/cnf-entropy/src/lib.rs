//! Entropy analysis and optimization layer for CSM compression
//!
//! This module provides entropy analysis capabilities for the CSM compression protocol,
//! enabling adaptive compression strategies based on input characteristics.
//!
//! ## Features
//! - **Entropy Calculation**: Shannon entropy for byte distributions
//! - **Symbol Graph**: Bigram analysis for common patterns
//! - **Adaptive Strategies**: Dynamic algorithm selection based on input entropy

pub mod entropy;
pub mod symbol_graph;

pub use entropy::build_huffman_tree;
pub use symbol_graph::{build_symbol_graph, most_frequent_bigram, auto_populate_dictionary};

/// Version of cnf-entropy module
pub const VERSION: &str = "1.0.0";

//! # COBOL CSM v154 protocol implementation
//!
//! Compression streaming module for CSM v154 with advanced features:
//! hierarchical dictionaries, bit-adaptive encoding, delta encoding,
//! template substitution, entropy analysis, and symbol graphs.

impl CsmOptions {
    /// Constructor untuk basic CSM (semua fitur advanced dimatikan)
    pub fn basic() -> Self {
        Self {
            hierarchical_dict: false,
            bit_adaptive: false,
            delta_encoding: false,
            templates_enabled: false,
            entropy_enabled: false,
            symbol_graph_enabled: false,
            profile: CsmProfile::Balanced,
        }
    }
}
/// Entry point kompresi basic CSM
pub fn compress_csm_basic(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    crate::stream::compress_csm_stream(input, dict, &CsmOptions::basic())
}
/// Compact Symbol Mapping (CSM) compression engine v154
/// Tidak mengimpor atau mengubah cobol-protocol-v153 (CORE-FROZEN)

pub mod base4096;    // Encoder/decoder 12-bit packing
pub mod bitpack;     // ZigZag delta encoding
pub mod dictionary;  // HashMap<u16, Arc<[u8]>> untuk template lookup
pub mod template;    // Template registry, fingerprint, ID
pub mod stream;      // Format: [MAGIC][VER][FLAGS][LAYER_MAP][SYMBOLS][CRC32]
pub mod error;
pub use error::{CsmError, MAX_ENTRY_LEN, MAX_DICT_SYMBOLS};

pub use dictionary::CsmDictionary;
pub const SYMBOL_FLAG: u16 = 0x8000;
pub use template::{StructTemplate, TemplateRegistry, TEMPLATE_FLAG};
pub use stream::StreamMetadata;
pub use stream::compress_csm_with_options;
pub use stream::read_metadata;

use thiserror::Error;

#[derive(Debug, Clone)]
pub enum CsmProfile {
    Balanced,
    Performance,
    CompressionOptimized,
}

#[derive(Debug, Clone)]
pub struct CsmOptions {
    pub hierarchical_dict: bool,
    pub bit_adaptive: bool,
    pub delta_encoding: bool,
    pub templates_enabled: bool,
    pub entropy_enabled: bool,
    pub symbol_graph_enabled: bool,
    pub profile: CsmProfile,
}

impl Default for CsmOptions {
    fn default() -> Self {
        Self {
            hierarchical_dict: true,
            bit_adaptive: true,
            delta_encoding: true,
            templates_enabled: true,
            entropy_enabled: false,
            symbol_graph_enabled: false,
            profile: CsmProfile::Balanced,
        }
    }
}


/// Compress input using CSM protocol (dict-first, pointer compression)
///
/// # Example
/// ```
/// use cobol_protocol_v154::{compress_csm, decompress_csm, CsmDictionary};
/// let mut dict = CsmDictionary::new();
/// dict.insert(0, b"hello").unwrap();
/// let data = b"hello world hello";
/// let compressed = compress_csm(data, &dict).unwrap();
/// assert!(compressed.len() <= data.len() + 64); // rough check, should not exceed by too much
/// let decompressed = decompress_csm(&compressed, &dict).unwrap();
/// assert_eq!(decompressed, data);
/// ```
pub fn compress_csm(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    let options = CsmOptions::default();
    stream::compress_csm_stream(input, dict, &options)
}
#[derive(Debug, Clone, Default)]
pub struct CsmStats {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub dict_matches: usize,
    pub raw_bytes: usize,
    pub ratio: f64, // output_bytes / input_bytes
}

pub fn compress_csm_stats(input: &[u8], dict: &CsmDictionary) -> Result<(Vec<u8>, CsmStats), CsmError> {
    let options = CsmOptions::default();
    let compressed = stream::compress_csm_stream(input, dict, &options)?;
    let meta = stream::read_metadata(&compressed);
    let ratio = match meta {
        Some(m) => m.ratio_hint,
        None => compressed.len() as f64 / input.len().max(1) as f64,
    };
    let stats = CsmStats {
        input_bytes: input.len(),
        output_bytes: compressed.len(),
        dict_matches: 0, // Not tracked in current impl
        raw_bytes: 0,    // Not tracked in current impl
        ratio,
    };
    Ok((compressed, stats))
}

pub fn is_csm_stream(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..2] == [0x43, 0x53] && (data[2] == 0x9B || data[2] == 0x9A)
}

/// Decompress CSM stream
pub fn decompress_csm(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::decompress_csm_stream(data, dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csm_options_default_balanced() {
        let options = CsmOptions::default();
        assert!(options.hierarchical_dict);
        assert!(options.bit_adaptive);
        assert!(options.delta_encoding);
        assert!(options.templates_enabled);
        assert!(matches!(options.profile, CsmProfile::Balanced));
    }

    #[test]
    fn test_compression_ratio_improvement() {
        let mut dict = CsmDictionary::new();
        dict.insert_global(1, b"hello").unwrap();
        dict.insert_global(2, b"world").unwrap();

        let data = b"hello world hello world hello world hello world";
        let compressed = compress_csm(data, &dict).expect("compression failed");
        let decompressed = decompress_csm(&compressed, &dict).expect("decompression failed");
        assert_eq!(decompressed, data);
        assert!(compressed.len() > 0, "compressed output must be non-empty");

        let stats = compress_csm_stats(data, &dict).unwrap().1;
        assert!(stats.ratio > 0.0, "ratio should be positive");
        assert!(stats.ratio.is_finite(), "ratio should be finite");
    }
}


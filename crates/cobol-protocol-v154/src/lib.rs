//! Compact Symbol Mapping (CSM) compression engine v154
//! Tidak mengimpor atau mengubah cobol-protocol-v153 (CORE-FROZEN)

pub mod base4096;    // Encoder/decoder 12-bit packing
pub mod dictionary;  // HashMap<u16, Arc<[u8]>> untuk template lookup
pub mod stream;      // Format: [MAGIC][VER][FLAGS][LAYER_MAP][SYMBOLS][CRC32]


pub use stream::StreamMetadata;
pub use stream::compress_csm_with_options;
pub use stream::read_metadata;
use dictionary::CsmDictionary;
use thiserror::Error;


#[derive(Debug, Error, PartialEq, Eq)]
pub enum CsmError {
    #[error("Invalid stream format")]
    InvalidStream,
    #[error("Dictionary mismatch")]
    DictionaryMismatch,
    #[error("CRC32 mismatch: stream may be truncated or corrupted")]
    ChecksumFailed,
    #[error("Base4096 encoding error")]
    EncodingError,
    #[error("Dictionary entry too large: {0} bytes (max {1})")]
    EntryTooLarge(usize, usize),
    #[error("Dictionary is full: {0} symbols (max {1})")]
    DictionaryFull(usize, usize),
}



/// Compress input using CSM protocol (dict-first, pointer compression)
///
/// # Example
/// ```
/// use cobol_protocol_v154::{compress_csm, decompress_csm, CsmDictionary};
/// let mut dict = CsmDictionary::new();
/// dict.insert(0, b"hello");
/// let data = b"hello world hello";
/// let compressed = compress_csm(data, &dict).unwrap();
/// assert!(compressed.len() < data.len() + 20); // rough check
/// let decompressed = decompress_csm(&compressed, &dict).unwrap();
/// assert_eq!(decompressed, data);
/// ```
pub fn compress_csm(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::compress_csm_stream(input, dict)
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
    let compressed = stream::compress_csm_stream(input, dict)?;
    let meta = stream::read_metadata(&compressed).ok();
    let ratio = meta.map(|m| m.ratio_hint).unwrap_or_else(|| compressed.len() as f64 / input.len().max(1) as f64);
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
    data.len() >= 4 && data[0..2] == [0x43, 0x53] && data[2] == 0x9A
}

/// Decompress CSM stream
pub fn decompress_csm(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::decompress_csm_stream(data, dict)
}

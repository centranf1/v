//! Compact Symbol Mapping (CSM) compression engine v154
//! Tidak mengimpor atau mengubah cobol-protocol-v153 (CORE-FROZEN)

pub mod base4096;    // Encoder/decoder 12-bit packing
pub mod dictionary;  // HashMap<u16, Arc<[u8]>> untuk template lookup
pub mod stream;      // Format: [MAGIC][VER][FLAGS][LAYER_MAP][SYMBOLS][CRC32]

use dictionary::CsmDictionary;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CsmError {
    #[error("Invalid stream format")]
    InvalidStream,
    #[error("Dictionary mismatch")]
    DictionaryMismatch,
    #[error("Checksum verification failed")]
    ChecksumFailed,
    #[error("Base4096 encoding error")]
    EncodingError,
}


/// Compress input using CSM protocol (dict-first, pointer compression)
pub fn compress_csm(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::compress_csm_stream(input, dict)
}

/// Decompress CSM stream
pub fn decompress_csm(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::decompress_csm_stream(data, dict)
}

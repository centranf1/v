//! Compact Symbol Mapping (CSM) compression engine
//! Layer: Protocol v154 (do not modify v153)

pub mod base4096;
pub mod dictionary;
pub mod stream;

use dictionary::CsmDictionary;
use thiserror::Error;

#[derive(Debug, Error)]
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

pub use dictionary::CsmDictionary;

/// Compress input using CSM protocol
pub fn compress_csm(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::compress_csm_stream(input, dict)
}

/// Decompress CSM stream
pub fn decompress_csm(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    stream::decompress_csm_stream(data, dict)
}

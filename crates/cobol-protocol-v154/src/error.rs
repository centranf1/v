// Error type for CsmDictionary
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CsmError {
        #[error("Dictionary overflow (max capacity exceeded)")]
        DictionaryOverflow,
    // Protocol errors
    #[error("Invalid stream format")]
    InvalidStream,
    #[error("Dictionary mismatch")]
    DictionaryMismatch,
    #[error("CRC32 mismatch: stream may be truncated or corrupted")]
    ChecksumFailed,
    #[error("Signature verification failed")]
    SignatureFailed,
    // Dictionary errors
    #[error("Dictionary full (max symbols reached)")]
    MaxSymbols,
    #[error("Entry too long (max {0} bytes)")]
    EntryTooLong(usize),
    #[error("Symbol already exists")]
    DuplicateSymbol,
    #[error("Other error: {0}")]
    Other(String),
}

pub const MAX_ENTRY_LEN: usize = 256;
pub const MAX_DICT_SYMBOLS: usize = 4096;

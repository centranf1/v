//! Convert utilities for CENTRA-NF

#[derive(Debug, Clone, PartialEq)]
pub enum CnfDataType {
    Int(i64),
    Float(f64),
    Str(String),
    Buffer(Vec<u8>),
    Bool(bool),
}

/// Konversi i64 ke buffer
pub fn i64_to_buffer(val: i64) -> Vec<u8> {
    val.to_be_bytes().to_vec()
}

/// Konversi buffer ke i64
pub fn buffer_to_i64(buf: &[u8]) -> Option<i64> {
    if buf.len() == 8 {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(buf);
        Some(i64::from_be_bytes(arr))
    } else {
        None
    }
}

/// Konversi f64 ke buffer
pub fn f64_to_buffer(val: f64) -> Vec<u8> {
    val.to_be_bytes().to_vec()
}

/// Konversi buffer ke f64
pub fn buffer_to_f64(buf: &[u8]) -> Option<f64> {
    if buf.len() == 8 {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(buf);
        Some(f64::from_be_bytes(arr))
    } else {
        None
    }
}

/// Konversi string ke buffer
pub fn str_to_buffer(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Konversi buffer ke string
pub fn buffer_to_str(buf: &[u8]) -> Option<String> {
    String::from_utf8(buf.to_vec()).ok()
}

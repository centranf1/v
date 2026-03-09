//! Format detection/validation utilities for CENTRA-NF

/// Deteksi format file dari magic bytes
pub fn detect_format(data: &[u8]) -> &'static str {
    if data.starts_with(b"{\"") || data.starts_with(b"[") {
        "JSON"
    } else if data.starts_with(b"<?xml") {
        "XML"
    } else if data.starts_with(b"\xFF\xD8\xFF") {
        "JPEG"
    } else if data.starts_with(b"\x00\x00\x00\x18ftypmp42") {
        "MP4"
    } else if data.starts_with(b"RIFF") && data[8..12] == *b"WAVE" {
        "WAV"
    } else if data.starts_with(b"PAR1") {
        "Parquet"
    } else {
        "Unknown"
    }
}

/// Validasi JSON
pub fn validate_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

/// Validasi CSV (sederhana)
pub fn validate_csv(s: &str) -> bool {
    s.lines().all(|line| line.split(',').count() > 1)
}

/// Validasi XML
pub fn validate_xml(s: &str) -> bool {
    quick_xml::de::from_str::<serde_json::Value>(s).is_ok()
}

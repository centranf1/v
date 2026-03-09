//! Buffer utilities for CENTRA-NF

/// Ukuran buffer dalam byte
pub fn size(buf: &[u8]) -> usize {
    buf.len()
}

/// Buffer kosong?
pub fn is_empty(buf: &[u8]) -> bool {
    buf.is_empty()
}

/// Buat buffer zero-filled
pub fn zeros(size: usize) -> Vec<u8> {
    vec![0; size]
}

/// Hex encode
pub fn hex_encode(buf: &[u8]) -> String {
    hex::encode(buf)
}

/// Hex decode
pub fn hex_decode(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(s)
}

/// Concat dua buffer
pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    [a, b].concat()
}

/// Slice buffer
pub fn slice(buf: &[u8], start: usize, len: usize) -> &[u8] {
    let end = usize::min(start + len, buf.len());
    &buf[start..end]
}

/// XOR dua buffer (panjang sama)
pub fn xor(a: &[u8], b: &[u8]) -> Option<Vec<u8>> {
    if a.len() != b.len() { return None; }
    Some(a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect())
}

/// Adler32 checksum
pub fn adler32(buf: &[u8]) -> u32 {
    use adler::Adler32;
    let mut adler = Adler32::new();
    adler.write_slice(buf);
    adler.checksum()
}

/// Cari pola dalam buffer
pub fn find_pattern(buf: &[u8], pat: &[u8]) -> Option<usize> {
    buf.windows(pat.len()).position(|window| window == pat)
}

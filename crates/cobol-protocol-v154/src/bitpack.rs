use std::io::{self, Read};

/// ZigZag transform signed integer ke unsigned untuk variable length.
pub fn zigzag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// ZigZag decode.
pub fn zigzag_decode(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

/// Hitung jumlah bit terendah yang cukup untuk menyimpan nilai maksimal.
pub fn calculate_min_bits(max_value: u64) -> u8 {
    if max_value == 0 {
        return 1;
    }
    let bits = 64 - max_value.leading_zeros();
    (bits as u8).min(16).max(1)
}

/// Writer bit-level untuk lebar dinamis 1..16 bit.
pub struct BitWriter {
    buffer: Vec<u8>,
    bit_accumulator: u32,
    bits_in_accumulator: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), bit_accumulator: 0, bits_in_accumulator: 0 }
    }

    pub fn write_bits(&mut self, mut value: u64, mut width: u8) -> io::Result<()> {
        if width == 0 || width > 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "width must be 1..16"));
        }
        if width < 64 && (value >> width) != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "value too large for width"));
        }

        // push to accumulator low-order bits
        self.bit_accumulator |= (value as u32) << self.bits_in_accumulator;
        self.bits_in_accumulator += width;

        while self.bits_in_accumulator >= 8 {
            self.buffer.push((self.bit_accumulator & 0xFF) as u8);
            self.bit_accumulator >>= 8;
            self.bits_in_accumulator -= 8;
        }

        Ok(())
    }

    pub fn flush(mut self) -> Vec<u8> {
        if self.bits_in_accumulator > 0 {
            self.buffer.push((self.bit_accumulator & 0xFF) as u8);
            self.bit_accumulator = 0;
            self.bits_in_accumulator = 0;
        }
        self.buffer
    }
}

/// Reader bit-level untuk lebar dinamis 1..16.
pub struct BitReader<'a> {
    data: &'a [u8],
    position: usize,
    bit_accumulator: u32,
    bits_in_accumulator: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0, bit_accumulator: 0, bits_in_accumulator: 0 }
    }

    pub fn read_bits(&mut self, width: u8) -> io::Result<u64> {
        if width == 0 || width > 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "width must be 1..16"));
        }

        while self.bits_in_accumulator < width {
            if self.position >= self.data.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough bits"));
            }
            let byte = self.data[self.position] as u32;
            self.bit_accumulator |= byte << self.bits_in_accumulator;
            self.bits_in_accumulator += 8;
            self.position += 1;
        }

        let mask = if width == 64 { !0u64 } else { (1u64 << width) - 1 };
        let result = (self.bit_accumulator as u64) & mask;
        self.bit_accumulator >>= width;
        self.bits_in_accumulator -= width;
        Ok(result)
    }
}

/// Delta compression untuk rangkaian integer signed.
/// Format: [base_i64_le (8 bytes)][width_u8][n_values_u32_le][delta_bits...]
pub fn encode_delta_i64(values: &[i64]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();

    if values.is_empty() {
        return Ok(out);
    }

    let base = values[0];
    out.extend_from_slice(&base.to_le_bytes());

    let mut prev = base;
    let mut max_zig = 0u64;
    let mut deltas = Vec::with_capacity(values.len().saturating_sub(1));

    for &v in &values[1..] {
        let delta = v.wrapping_sub(prev);
        let zig = zigzag_encode(delta);
        if zig > max_zig {
            max_zig = zig;
        }
        deltas.push(zig);
        prev = v;
    }

    let width = calculate_min_bits(max_zig);
    out.push(width);
    out.extend_from_slice(&(values.len() as u32).to_le_bytes());

    let mut writer = BitWriter::new();
    for zig in deltas {
        writer.write_bits(zig, width)?;
    }
    out.extend_from_slice(&writer.flush());
    Ok(out)
}

pub fn decode_delta_i64(encoded: &[u8]) -> io::Result<Vec<i64>> {
    if encoded.len() < 8 + 1 + 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "encoded delta too short"));
    }

    let base = i64::from_le_bytes(encoded[0..8].try_into().unwrap());
    let width = encoded[8];
    let len = u32::from_le_bytes(encoded[9..13].try_into().unwrap()) as usize;
    if len == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "length must be at least 1"));
    }

    let mut out = Vec::with_capacity(len);
    out.push(base);

    if len == 1 {
        return Ok(out);
    }

    let mut reader = BitReader::new(&encoded[13..]);
    let mut prev = base;

    for _ in 1..len {
        let zig = reader.read_bits(width)?;
        let delta = zigzag_decode(zig);
        let next = prev.wrapping_add(delta);
        out.push(next);
        prev = next;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_roundtrip() {
        let xs = [-1, -2, -3, 0, 1, 2, 12345678];
        for &x in &xs {
            assert_eq!(zigzag_decode(zigzag_encode(x)), x);
        }
    }

    #[test]
    fn test_calculate_min_bits_various() {
        assert_eq!(calculate_min_bits(0), 1);
        assert_eq!(calculate_min_bits(1), 1);
        assert_eq!(calculate_min_bits(2), 2);
        assert_eq!(calculate_min_bits(3), 2);
        assert_eq!(calculate_min_bits(255), 8);
        assert_eq!(calculate_min_bits(65535), 16);
    }

    #[test]
    fn test_delta_encode_decode() {
        let values: Vec<i64> = vec![100, 105, 108, 95, 1000, 1005];
        let encoded = encode_delta_i64(&values).unwrap();
        let restored = decode_delta_i64(&encoded).unwrap();
        assert_eq!(values, restored);
    }
}

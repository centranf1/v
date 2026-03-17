
// Base4096 encoding/decoding (12-bit symbols)
// Each symbol: u16 (0..4095)

pub fn encode(input: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for &b in input {
        acc = (acc << 8) | b as u32;
        bits += 8;
        while bits >= 12 {
            bits -= 12;
            out.push(((acc >> bits) & 0xFFF) as u16);
        }
    }
    if bits > 0 {
        out.push(((acc << (12 - bits)) & 0xFFF) as u16);
    }
    out
}

pub fn decode(symbols: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for &sym in symbols {
        acc = (acc << 12) | (sym as u32 & 0xFFF);
        bits += 12;
        while bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    out
}

/// Packs tokens (u16, 0..4095) into bytes (12 bits per token)
#[inline]
pub fn pack_tokens(tokens: &[u16]) -> Vec<u8> {
    let byte_count = (tokens.len() * 12 + 7) / 8;
    let mut out = Vec::with_capacity(byte_count);
    out.reserve_exact(byte_count);
    pack_tokens_into(tokens, &mut out);
    out
}

/// Streaming variant: appends packed bytes into existing Vec<u8>
#[inline]
pub fn pack_tokens_into(tokens: &[u16], out: &mut Vec<u8>) {
    let byte_count = (tokens.len() * 12 + 7) / 8;
    out.reserve_exact(byte_count);
    let mut i = 0;
    // Batch: 8 tokens = 12 bytes
    while i + 8 <= tokens.len() {
        let t = &tokens[i..i+8];
        let v = [
            ((t[0] >> 4) as u8),
            (((t[0] & 0xF) << 4 | (t[1] >> 8) as u16) as u8),
            (t[1] & 0xFF) as u8,
            ((t[2] >> 4) as u8),
            (((t[2] & 0xF) << 4 | (t[3] >> 8) as u16) as u8),
            (t[3] & 0xFF) as u8,
            ((t[4] >> 4) as u8),
            (((t[4] & 0xF) << 4 | (t[5] >> 8) as u16) as u8),
            (t[5] & 0xFF) as u8,
            ((t[6] >> 4) as u8),
            (((t[6] & 0xF) << 4 | (t[7] >> 8) as u16) as u8),
            (t[7] & 0xFF) as u8,
        ];
        out.extend_from_slice(&v);
        i += 8;
    }
    // Fallback: 2 tokens = 3 bytes
    while i + 2 <= tokens.len() {
        let t0 = tokens[i] & 0xFFF;
        let t1 = tokens[i+1] & 0xFFF;
        out.push((t0 >> 4) as u8);
        out.push(((t0 & 0xF) << 4 | (t1 >> 8) as u16) as u8);
        out.push((t1 & 0xFF) as u8);
        i += 2;
    }
    // Remainder: 1 token (12 bits)
    if i < tokens.len() {
        let t0 = tokens[i] & 0xFFF;
        out.push((t0 >> 4) as u8);
        out.push(((t0 & 0xF) << 4) as u8);
    }
}

/// Unpacks bytes into tokens (u16, 0..4095)
pub fn unpack_tokens(bytes: &[u8]) -> Vec<u16> {
    let max_tokens = (bytes.len() * 8) / 12;
    let mut out = Vec::with_capacity(max_tokens);
    out.reserve_exact(max_tokens);
    let mut i = 0;
    // Batch: 12 bytes = 8 tokens
    while i + 12 <= bytes.len() {
        let b = &bytes[i..i+12];
        out.push(((b[0] as u16) << 4) | ((b[1] as u16) >> 4));
        out.push((((b[1] as u16 & 0xF) << 8) | b[2] as u16));
        out.push(((b[3] as u16) << 4) | ((b[4] as u16) >> 4));
        out.push((((b[4] as u16 & 0xF) << 8) | b[5] as u16));
        out.push(((b[6] as u16) << 4) | ((b[7] as u16) >> 4));
        out.push((((b[7] as u16 & 0xF) << 8) | b[8] as u16));
        out.push(((b[9] as u16) << 4) | ((b[10] as u16) >> 4));
        out.push((((b[10] as u16 & 0xF) << 8) | b[11] as u16));
        i += 12;
    }
    // Fallback: 3 bytes = 2 tokens
    while i + 3 <= bytes.len() {
        let t0 = ((bytes[i] as u16) << 4) | ((bytes[i+1] as u16) >> 4);
        let t1 = (((bytes[i+1] as u16 & 0xF) << 8) | bytes[i+2] as u16);
        out.push(t0);
        out.push(t1);
        i += 3;
    }
    // Remainder: 2 bytes = 1 token
    if i + 2 == bytes.len() {
        let t0 = ((bytes[i] as u16) << 4) | ((bytes[i+1] as u16) >> 4);
        out.push(t0);
    }
    out
}

/// Returns true if byte count is consistent with valid pack_tokens output
pub fn validate_packed(bytes: &[u8]) -> bool {
    let bits = bytes.len() * 8;
    let rem = bits % 12;
    match rem {
        0 => true,
        4 => bytes.len() >= 2 && (bytes.len() - 2) % 3 == 0,
        8 => bytes.len() >= 3 && (bytes.len() - 3) % 3 == 0,
        _ => false,
    }
}

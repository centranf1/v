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

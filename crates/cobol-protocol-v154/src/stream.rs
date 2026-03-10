use crate::dictionary::CsmDictionary;
use crate::CsmError;
use crc32fast::Hasher as Crc32Hasher;

const MAGIC: [u8; 2] = [0x43, 0x53]; // "CS"
const VERSION: u8 = 0x9A;
const HEADER_LEN: usize = 2 + 1 + 1 + 8; // magic + version + flags + layer_map

pub fn compress_csm_stream(input: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.push(0u8); // flags: 0x01 = dict used
    out.extend_from_slice(&[0u8; 8]); // layer_map reserved

    let mut tokens: Vec<u16> = Vec::new();
    let mut i = 0usize;
    let mut dict_used = false;

    while i < input.len() {
        // Try longest dictionary match
        let mut best = (0usize, None::<u16>);
        for sym in 0u16..4096 {
            if let Some(entry) = dict.lookup(sym) {
                let end = i + entry.len();
                if !entry.is_empty() && end <= input.len() && &input[i..end] == entry && entry.len() > best.0 {
                    best = (entry.len(), Some(sym));
                }
            }
        }
        if let Some(sym_id) = best.1 {
            tokens.push(0x8000 | (sym_id & 0x7FFF)); // dict pointer token
            i += best.0;
            dict_used = true;
        } else {
            tokens.push(input[i] as u16 & 0x0FFF); // raw byte token
            i += 1;
        }
    }

    if dict_used { out[3] = 0x01; }
    for t in &tokens { out.extend_from_slice(&t.to_be_bytes()); }

    let mut crc = Crc32Hasher::new();
    crc.update(&out);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
    Ok(out)
}

pub fn decompress_csm_stream(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    if data.len() < HEADER_LEN + 4 { return Err(CsmError::InvalidStream); }
    if data[0..2] != MAGIC || data[2] != VERSION { return Err(CsmError::InvalidStream); }
    let dict_used = data[3] & 0x01 != 0;

    let crc_offset = data.len() - 4;
    let (content, crc_bytes) = data.split_at(crc_offset);
    let mut crc = Crc32Hasher::new();
    crc.update(content);
    if crc_bytes != crc.finalize().to_be_bytes() { return Err(CsmError::ChecksumFailed); }

    let symbol_bytes = &content[HEADER_LEN..];
    if symbol_bytes.len() % 2 != 0 { return Err(CsmError::InvalidStream); }

    let mut out = Vec::new();
    for chunk in symbol_bytes.chunks(2) {
        let token = u16::from_be_bytes([chunk[0], chunk[1]]);
        if dict_used && token & 0x8000 != 0 {
            let entry = dict.lookup(token & 0x7FFF).ok_or(CsmError::DictionaryMismatch)?;
            out.extend_from_slice(entry);
        } else {
            out.push((token & 0x00FF) as u8);
        }
    }
    Ok(out)
}

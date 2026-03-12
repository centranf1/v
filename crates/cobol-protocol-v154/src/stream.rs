#[derive(Debug, Clone, Copy, Default)]
pub struct StreamMetadata {
    pub magic: [u8; 2],
    pub version: u8,
    pub flags: u8,
    pub orig_size: u32,
    pub ratio_hint: f64,
}

pub fn read_metadata(data: &[u8]) -> Option<StreamMetadata> {
    if data.len() < HEADER_LEN + 4 { return None; }
    let magic = [data[0], data[1]];
    let version = data[2];
    let flags = data[3];
    // orig_size: baca dari header (LE)
    let orig_size = u32::from_le_bytes([
        data[HEADER_LEN],
        data[HEADER_LEN + 1],
        data[HEADER_LEN + 2],
        data[HEADER_LEN + 3],
    ]);
    let ratio_hint = data.len() as f64 / orig_size.max(1) as f64;
    Some(StreamMetadata { magic, version, flags, orig_size, ratio_hint })
}

pub fn compress_csm_with_options(input: &[u8], dict: &CsmDictionary, _options: ()) -> Result<Vec<u8>, CsmError> {
    compress_csm_stream(input, dict)
}
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
    // Tambahkan orig_size (4 byte LE)
    let orig_size = input.len() as u32;
    out.extend_from_slice(&orig_size.to_le_bytes());

    // Pre-allocate tokens: after lazy matching, token count is lower
    let mut tokens: Vec<u16> = Vec::with_capacity(input.len() / 3 + 32);
    let mut i = 0usize;
    let mut dict_used = false;
    let dict_syms: Vec<u16> = dict
        .iter()
        .map(|(k, _)| k)
        .collect();

    while i < input.len() {
        // Find all candidates, sorted by entry length descending
        let mut candidates: Vec<(usize, u16)> = dict_syms
            .iter()
            .filter_map(|&sym| {
                if let Some(entry) = dict.lookup(sym) {
                    let end = i + entry.len();
                    if !entry.is_empty() && end <= input.len() && &input[i..end] == entry {
                        return Some((entry.len(), sym));
                    }
                }
                None
            })
            .collect();
        candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        if !candidates.is_empty() {
            eprintln!("[compress] i={} candidates: {:?}", i, candidates);
        }

        let (best_len, best_sym) = candidates.first().map(|(len, sym)| (*len, *sym)).unwrap_or((0, 0));

        // LAZY MATCHING: only skip match if next position has strictly longer match
        if best_len >= 2 && i + 1 < input.len() {
            let mut next_candidates: Vec<(usize, u16)> = dict_syms
                .iter()
                .filter_map(|&sym| {
                    if let Some(entry) = dict.lookup(sym) {
                        let end = i + 1 + entry.len();
                        if !entry.is_empty() && end <= input.len() && &input[i + 1..end] == entry {
                            return Some((entry.len(), sym));
                        }
                    }
                    None
                })
                .collect();
            next_candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));
            if let Some((next_len, _)) = next_candidates.first() {
                if *next_len > best_len {
                    // Emit raw byte at i, use longer match at i+1
                    tokens.push(input[i] as u16 & 0x0FFF);
                    i += 1;
                    continue;
                }
            }
        }
        // If best_len >= 2, always use dictionary match
        if best_len >= 2 {
            eprintln!("[compress] i={} use dict sym={} len={}", i, best_sym, best_len);
            tokens.push(0x8000 | (best_sym & 0x7FFF));
            i += best_len;
            dict_used = true;
        } else {
            tokens.push(input[i] as u16 & 0x0FFF);
            i += 1;
        }
    }

    if dict_used {
        out[3] = 0x01;
    }

    // Streaming pack tokens into out
    use crate::base4096::pack_tokens_into;
    pack_tokens_into(&tokens, &mut out);

    let mut crc = Crc32Hasher::new();
    crc.update(&out);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
    Ok(out)
}

pub fn decompress_csm_stream(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    const HEADER_WITH_ORIG: usize = HEADER_LEN + 4; // +4 untuk orig_size
    if data.len() < HEADER_WITH_ORIG + 4 { return Err(CsmError::InvalidStream); }
    if data[0..2] != MAGIC || data[2] != VERSION { return Err(CsmError::InvalidStream); }
    let dict_used = data[3] & 0x01 != 0;

    let crc_offset = data.len() - 4;
    let (content, crc_bytes) = data.split_at(crc_offset);
    let mut crc = Crc32Hasher::new();
    crc.update(content);
    if crc_bytes != crc.finalize().to_be_bytes() { return Err(CsmError::ChecksumFailed); }

    // symbol_bytes dimulai setelah header+orig_size
    let symbol_bytes = &content[HEADER_LEN + 4..];
    use crate::base4096::unpack_tokens;
    let tokens = unpack_tokens(symbol_bytes);
    let mut out = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if dict_used && token & 0x8000 != 0 {
            let sym = token & 0x7FFF;
            eprintln!("[decompress] token[{}]=0x{:04X} (dict sym={})", i, token, sym);
            match dict.lookup(sym) {
                Some(entry) => {
                    eprintln!("[decompress]  -> entry.len={} entry[..8]={:?}", entry.len(), &entry[..entry.len().min(8)]);
                    out.extend_from_slice(entry);
                },
                None => {
                    eprintln!("[decompress]  -> symbol {} NOT FOUND in dictionary!", sym);
                    return Err(CsmError::DictionaryMismatch);
                }
            }
        } else {
            eprintln!("[decompress] token[{}]=0x{:04X} (raw byte={})", i, token, token & 0x00FF);
            out.push((token & 0x00FF) as u8);
        }
    }
    Ok(out)
}

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

    // Pre-allocate tokens: after lazy matching, token count is lower
    let mut tokens: Vec<u16> = Vec::with_capacity(input.len() / 3 + 32);
    let mut i = 0usize;
    let mut dict_used = false;
    let dict_syms: Vec<u16> = dict
        .map
        .iter()
        .map(|(k, _)| *k)
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

        let (best_len, best_sym) = candidates.first().map(|(len, sym)| (*len, *sym)).unwrap_or((0, 0));

        // LAZY MATCHING: peek at i+1 for longer match
        let mut use_dict = false;
        let mut chosen_len = best_len;
        let mut chosen_sym = best_sym;
        if best_len >= 2 && i + 1 < input.len() {
            // Check if next position has a longer match
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
            use_dict = true;
        } else if best_len >= 2 {
            use_dict = true;
        }

        // GREEDY FALLBACK: if best_len == 1, treat as no match
        if use_dict && chosen_len >= 2 {
            tokens.push(0x8000 | (chosen_sym & 0x7FFF));
            i += chosen_len;
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

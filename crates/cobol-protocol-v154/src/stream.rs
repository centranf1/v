use crate::base4096;
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
    out.push(0); // flags (reserved)
    out.extend_from_slice(&[0u8; 8]); // layer_map (reserved)
    let symbols = base4096::encode(input);
    for sym in &symbols {
        out.extend_from_slice(&sym.to_be_bytes());
    }
    let mut crc = Crc32Hasher::new();
    crc.update(&out);
    let crc32 = crc.finalize();
    out.extend_from_slice(&crc32.to_be_bytes());
    Ok(out)
}

pub fn decompress_csm_stream(data: &[u8], _dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    if data.len() < HEADER_LEN + 4 {
        return Err(CsmError::InvalidStream);
    }
    if &data[0..2] != MAGIC {
        return Err(CsmError::InvalidStream);
    }
    if data[2] != VERSION {
        return Err(CsmError::InvalidStream);
    }
    let crc_offset = data.len() - 4;
    let (content, crc_bytes) = data.split_at(crc_offset);
    let mut crc = Crc32Hasher::new();
    crc.update(content);
    let crc32 = crc.finalize();
    if crc_bytes != crc32.to_be_bytes() {
        return Err(CsmError::ChecksumFailed);
    }
    let symbol_bytes = &content[HEADER_LEN..];
    if symbol_bytes.len() % 2 != 0 {
        return Err(CsmError::InvalidStream);
    }
    let mut symbols = Vec::with_capacity(symbol_bytes.len() / 2);
    for chunk in symbol_bytes.chunks(2) {
        let sym = u16::from_be_bytes([chunk[0], chunk[1]]);
        if sym > 4095 {
            return Err(CsmError::EncodingError);
        }
        symbols.push(sym);
    }
    Ok(base4096::decode(&symbols))
}

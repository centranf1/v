use crate::template::TemplateFieldType;
use cnf_quantum::{dilithium_sign, dilithium_verify, DilithiumSignature};
use cnf_security::sha256_hex;
/// Menandatangani CSM frame dengan ML-DSA (Dilithium3)
pub fn sign_csm_frame(frame: &[u8], signing_key: &[u8]) -> Result<Vec<u8>, CsmError> {
    let sig = dilithium_sign(signing_key, frame)
        .map_err(|_| CsmError::SignatureFailed)?;
    let mut out = Vec::with_capacity(frame.len() + sig.signature_bytes.len());
    out.extend_from_slice(frame);
    out.extend_from_slice(&sig.signature_bytes);
    Ok(out)
}

/// Verifikasi signature ML-DSA pada CSM frame
pub fn verify_csm_frame(frame_with_sig: &[u8], verification_key: &[u8]) -> Result<bool, CsmError> {
    // CRC selalu 4 byte terakhir sebelum signature
    if frame_with_sig.len() < 4 + 2420 { return Err(CsmError::InvalidStream); }
    let frame_len = frame_with_sig.len() - 2420;
    let frame = &frame_with_sig[..frame_len];
    let sig_bytes = &frame_with_sig[frame_len..];
    let sig = DilithiumSignature {
        algorithm: "ML-DSA-65".to_string(),
        signature_bytes: sig_bytes.to_vec(),
        message_hash: cnf_security::sha256_hex(frame),
    };
    dilithium_verify(verification_key, frame, &sig)
        .map_err(|_| CsmError::SignatureFailed)
}
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

use crate::bitpack::{decode_delta_i64, encode_delta_i64};
use crate::dictionary::CsmDictionary;
use crate::template::{StructTemplate, TemplateRegistry};
use crate::CsmError;
use crc32fast::Hasher as Crc32Hasher;
use std::borrow::Cow;

const MAGIC: [u8; 2] = [0x43, 0x53]; // "CS"
const VERSION: u8 = 0x9B; // v154 current
const HEADER_LEN: usize = 2 + 1 + 1 + 8; // magic + version + flags + layer_map

use crate::SYMBOL_FLAG;
const SYMBOL_MASK: u16 = 0x7FF; // 11-bit symbol index

fn preprocess_input<'a>(input: &'a [u8], options: &crate::CsmOptions) -> (Cow<'a, [u8]>, u8) {
    // Option delta encoding: when data is i64 aligned and sequence is quasi-linear.
    if options.delta_encoding && input.len() % 8 == 0 && input.len() >= 16 {
        let values: Vec<i64> = input
            .chunks_exact(8)
            .map(|c| i64::from_le_bytes(c.try_into().unwrap()))
            .collect();

        let max_delta = values
            .windows(2)
            .map(|w| w[1].wrapping_sub(w[0]).unsigned_abs())
            .max()
            .unwrap_or(0);

        if max_delta < 100_000 {
            if let Ok(delta_bytes) = encode_delta_i64(&values) {
                if delta_bytes.len() < input.len() {
                    return (Cow::Owned(delta_bytes), 0x04);
                }
            }
        }
    }

    (Cow::Borrowed(input), 0)
}

fn tokenize_and_pack(input: &[u8], dict: &CsmDictionary, options: &crate::CsmOptions) -> Result<(Vec<u16>, bool), CsmError> {
    let mut tokens: Vec<u16> = Vec::with_capacity(input.len() / 2 + 64);
    let mut i = 0usize;
    let mut dict_used = false;

    // Template match (deterministik): jika templates_enabled, deteksi template
    if options.templates_enabled {
        // Contoh: template default, bisa diubah sesuai kebutuhan
        let template = crate::template::StructTemplate::new(vec![crate::template::TemplateFieldType::IntegerSigned, crate::template::TemplateFieldType::Utf8String]);
        let mut registry = crate::template::TemplateRegistry::new();
        let template_id = registry.register(&template).map_err(|e| CsmError::InvalidStream)?;
        tokens.push(crate::template::template_token(template_id));
    }

    while i < input.len() {
        let mut candidates: Vec<(usize, u16)> = dict
            .candidates_for_byte(input[i])
            .into_iter()
            .filter_map(|(sym, entry)| {
                let end = i + entry.len();
                if entry.len() >= 2 && end <= input.len() && &input[i..end] == entry {
                    Some((entry.len(), sym))
                } else {
                    None
                }
            })
            .collect();
        candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        let (best_len, best_sym) = candidates.first().map(|(len, sym)| (*len, *sym)).unwrap_or((0, 0));

        if best_len >= 2 && i + 1 < input.len() {
            let mut next_candidates: Vec<(usize, u16)> = dict
                .candidates_for_byte(input[i + 1])
                .into_iter()
                .filter_map(|(sym, entry)| {
                    let end = i + 1 + entry.len();
                    if entry.len() >= 2 && end <= input.len() && &input[i + 1..end] == entry {
                        Some((entry.len(), sym))
                    } else {
                        None
                    }
                })
                .collect();
            next_candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0));
            if let Some((next_len, _)) = next_candidates.first() {
                if *next_len > best_len {
                    tokens.push(input[i] as u16);
                    i += 1;
                    continue;
                }
            }
        }

        if best_len >= 2 {
            if best_sym > SYMBOL_MASK {
                return Err(CsmError::DictionaryOverflow);
            }
            tokens.push(SYMBOL_FLAG | (best_sym & SYMBOL_MASK));
            i += best_len;
            dict_used = true;
        } else {
            tokens.push(input[i] as u16);
            i += 1;
        }
    }

    // Hook untuk entropy dan symbol graph (future):
    if options.entropy_enabled {
        // entropy pass (Huffman) akan dipanggil di sini
    }
    if options.symbol_graph_enabled {
        // symbol graph (bigram auto-populate) akan dipanggil di sini
    }

    Ok((tokens, dict_used))
}

pub fn compress_csm_with_options(input: &[u8], dict: &CsmDictionary, options: &crate::CsmOptions) -> Result<Vec<u8>, CsmError> {
    compress_csm_stream(input, dict, options)
}

pub fn compress_csm_stream(input: &[u8], dict: &CsmDictionary, options: &crate::CsmOptions) -> Result<Vec<u8>, CsmError> {
    let (source, preprocessing_flag) = preprocess_input(input, options);

    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    let mut flags = 0u8;
    if options.hierarchical_dict { flags |= 0x10; }
    if options.bit_adaptive { flags |= 0x02; }
    if options.templates_enabled { flags |= 0x08; }
    if preprocessing_flag != 0 { flags |= preprocessing_flag; }
    out.push(flags);
    // Isi layer_map otomatis: [template, dict, delta, bit_adaptive, hierarchical, reserved]
    let mut layer_map = [0u8; 8];
    if options.templates_enabled { layer_map[0] = 1; }
    if dict_used { layer_map[1] = 1; }
    if options.delta_encoding { layer_map[2] = 1; }
    if options.bit_adaptive { layer_map[3] = 1; }
    if options.hierarchical_dict { layer_map[4] = 1; }
    out.extend_from_slice(&layer_map);
    // Tambahkan orig_size (4 byte LE)
    let orig_size = input.len() as u32;
    out.extend_from_slice(&orig_size.to_le_bytes());

    // Tokenize and pack from preprocessed input
    let (mut tokens, dict_used) = tokenize_and_pack(&source, dict, options)?;

    // Integrasi TemplateRegistry: encode template token jika templates_enabled
    if options.templates_enabled {
        // Contoh: template default, bisa diubah sesuai kebutuhan
        let template = StructTemplate::new(vec![TemplateFieldType::IntegerSigned, TemplateFieldType::Utf8String]);
        let mut registry = TemplateRegistry::new();
        let template_id = registry.register(&template).map_err(|_e| CsmError::InvalidStream)?;
        tokens.insert(0, crate::template::template_token(template_id));
    }

    // Set layer_map[1] jika dict_used
    if dict_used {
        layer_map[1] = 1;
    }

    // Bit-adaptive encoding: scan distribusi token, hitung bit_width
    let max_token = tokens.iter().copied().max().unwrap_or(0) as u64;
    let bit_width = crate::bitpack::calculate_min_bits(max_token);
    layer_map[0] = bit_width;

    // Encode tokens dengan BitWriter
    let mut bit_writer = crate::bitpack::BitWriter::new();
    for token in &tokens {
        bit_writer.write_bits(*token as u64, bit_width).unwrap();
    }
    let packed = bit_writer.flush();
    out.extend_from_slice(&packed);

    let mut crc = Crc32Hasher::new();
    crc.update(&out);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
    Ok(out)
}

pub fn decompress_csm_stream(data: &[u8], dict: &CsmDictionary) -> Result<Vec<u8>, CsmError> {
    const HEADER_WITH_ORIG: usize = HEADER_LEN + 4; // +4 untuk orig_size
    if data.len() < HEADER_WITH_ORIG + 4 { return Err(CsmError::InvalidStream); }
    if data[0..2] != MAGIC || !(data[2] == VERSION || data[2] == 0x9A) { return Err(CsmError::InvalidStream); }
    let dict_used = data[3] & 0x01 != 0;
    let _has_delta = data[3] & 0x04 != 0;
    let _has_templates = data[3] & 0x08 != 0;

    // Baca layer_map dari header
    let layer_map = &data[4..12];
    let bit_width = layer_map[0];

    let crc_offset = data.len() - 4;
    let (content, crc_bytes) = data.split_at(crc_offset);
    let mut crc = Crc32Hasher::new();
    crc.update(content);
    if crc_bytes != crc.finalize().to_be_bytes() { return Err(CsmError::ChecksumFailed); }

    // symbol_bytes dimulai setelah header+orig_size
    let symbol_bytes = &content[HEADER_LEN + 4..];
    // Validasi symbol_bytes jika perlu
    let mut reader = crate::bitpack::BitReader::new(symbol_bytes);
    let mut tokens = Vec::new();
    // Estimasi jumlah token: (symbol_bytes.len() * 8) / bit_width
    let estimated_tokens = (symbol_bytes.len() * 8) / (bit_width.max(1) as usize);
    tokens.reserve_exact(estimated_tokens);
    // Baca token dari BitReader
    while let Ok(token) = reader.read_bits(bit_width) {
        tokens.push(token as u16);
    }
    let mut out = Vec::new();

    // Implementasi skip-layer logic
    let mut i = 0;
    if layer_map[0] == 1 && _has_templates {
        // Decode template token
        if let Some(template_id) = crate::template::decode_template_token(tokens[0]) {
            log::trace!("[csm::decompress] template token found: id={}", template_id);
            // TemplateRegistry bisa digunakan di sini jika ingin lookup
            i += 1; // skip template token
        }
    }

    for (idx, token) in tokens.iter().enumerate().skip(i) {
        if layer_map[1] == 1 && dict_used && token & SYMBOL_FLAG != 0 {
            let sym = token & SYMBOL_MASK;
            log::trace!("[csm::decompress] token[{}]=0x{:04X} (dict sym={})", idx, token, sym);
            match dict.lookup(sym) {
                Some(entry) => {
                    log::trace!("[csm::decompress] dict entry.len={}", entry.len());
                    out.extend_from_slice(entry);
                },
                None => {
                    log::trace!("[csm::decompress] dict symbol {} NOT FOUND", sym);
                    return Err(CsmError::DictionaryMismatch);
                }
            }
        } else {
            log::trace!("[csm::decompress] token[{}]=0x{:04X} (raw byte={})", idx, token, token & 0x00FF);
            out.push((token & 0x00FF) as u8);
        }
    }

    if layer_map[2] == 1 && _has_delta {
        let decoded = decode_delta_i64(&out)
            .map_err(|_| CsmError::InvalidStream)?;
        let mut expanded = Vec::with_capacity(decoded.len() * 8);
        for value in decoded {
            expanded.extend_from_slice(&value.to_le_bytes());
        }
        return Ok(expanded);
    }

    Ok(out)
}

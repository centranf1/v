use cobol_protocol_v154::{base4096, dictionary::CsmDictionary, compress_csm, decompress_csm, CsmError};

#[test]
fn test_base4096_roundtrip() {
    let data = b"abc123xyz";
    let encoded = base4096::encode(data);
    let decoded = base4096::decode(&encoded);
    assert_eq!(data, &decoded[..]);
}

#[test]
fn test_base4096_determinism() {
    let data = b"repeatable";
    let a = base4096::encode(data);
    let b = base4096::encode(data);
    assert_eq!(a, b);
}

#[test]
fn test_dictionary_insert_lookup() {
    let mut dict = CsmDictionary::new();
    dict.insert(42, b"foobar").unwrap();
    assert_eq!(dict.lookup(42), Some(&b"foobar"[..]));
}

#[test]
fn test_dictionary_checksum_verification() {
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"a").unwrap();
    dict.insert(2, b"b").unwrap();
    let orig = dict.checksum;
    assert!(dict.verify_checksum());
    dict.insert(3, b"c").unwrap();
    assert_ne!(dict.checksum, orig);
}

#[test]
fn test_stream_header_validation() {
    let dict = CsmDictionary::new();
    let options = cobol_protocol_v154::CsmOptions::default();
    let data = b"testdata";
    let compressed = cobol_protocol_v154::stream::compress_csm_stream(data, &dict, &options).unwrap();
    assert_eq!(&compressed[0..2], b"CS");
    assert!(compressed[2] == 0x9B || compressed[2] == 0x9A);
}

#[test]
fn test_crc32_tamper_detection() {
    let dict = CsmDictionary::new();
    let data = b"tamper";
    let mut compressed = compress_csm(data, &dict).unwrap();
    let len = compressed.len();
    compressed[len - 5] ^= 0xFF; // corrupt a byte before CRC
    let res = decompress_csm(&compressed, &dict);
    assert!(matches!(res, Err(CsmError::ChecksumFailed)));
}

#[test]
fn test_compress_decompress_roundtrip() {
    let dict = CsmDictionary::new();
    let data = b"roundtrip test data";
    let compressed = compress_csm(data, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(data, &decompressed[..]);
}

#[test]
fn test_dictionary_pointer_compression_ratio() {
    let mut dict = CsmDictionary::new();
    for i in 0..32u16 {
        dict.insert(i, &[i as u8; 4]).unwrap();
    }
    let data = [0u8; 128];
    let compressed = compress_csm(&data, &dict).unwrap();
    // Ratio: compressed size should be less than input + header
    assert!(compressed.len() < data.len() + 16);
}

#[test]
fn batch_pack_8tokens_at_once() {
    use cobol_protocol_v154::base4096::{pack_tokens, unpack_tokens};
    let tokens: Vec<u16> = (0..8).map(|i| (i * 123) & 0xFFF).collect();
    let packed = pack_tokens(&tokens);
    let unpacked = unpack_tokens(&packed);
    assert_eq!(tokens, unpacked);
}

#[test]
fn pack_tokens_into_appends_correctly() {
    use cobol_protocol_v154::base4096::{pack_tokens, pack_tokens_into, unpack_tokens};
    let tokens1: Vec<u16> = (0..4).map(|i| (i * 99) & 0xFFF).collect();
    let tokens2: Vec<u16> = (4..12).map(|i| (i * 99) & 0xFFF).collect();
    let mut buf = pack_tokens(&tokens1);
    let orig_len = buf.len();
    pack_tokens_into(&tokens2, &mut buf);
    let unpacked = unpack_tokens(&buf);
    let mut expected = tokens1;
    expected.extend(tokens2.clone());
    assert_eq!(unpacked, expected);
    assert_eq!(buf.len(), orig_len + pack_tokens(&tokens2).len());
}

#[test]
fn validate_packed_rejects_invalid_length() {
    use cobol_protocol_v154::base4096::{pack_tokens, validate_packed};
    let tokens: Vec<u16> = (0..5).map(|i| (i * 77) & 0xFFF).collect();
    let mut packed = pack_tokens(&tokens);
    // Valid length
    assert!(validate_packed(&packed));
    // Add 1 extra byte (invalid)
    packed.push(0xFF);
    assert!(!validate_packed(&packed));
}

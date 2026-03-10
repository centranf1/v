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
    dict.insert(42, b"foobar");
    assert_eq!(dict.lookup(42), Some(&b"foobar"[..]));
}

#[test]
fn test_dictionary_checksum_verification() {
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"a");
    dict.insert(2, b"b");
    let orig = dict.checksum();
    assert!(dict.verify_checksum());
    dict.insert(3, b"c");
    assert_ne!(dict.checksum(), orig);
}

#[test]
fn test_stream_header_validation() {
    let dict = CsmDictionary::new();
    let data = b"testdata";
    let compressed = compress_csm(data, &dict).unwrap();
    assert_eq!(&compressed[0..2], b"CS");
    assert_eq!(compressed[2], 0x9A);
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
        dict.insert(i, &[i as u8; 4]);
    }
    let data = [0u8; 128];
    let compressed = compress_csm(&data, &dict).unwrap();
    // Ratio: compressed size should be less than input + header
    assert!(compressed.len() < data.len() + 16);
}

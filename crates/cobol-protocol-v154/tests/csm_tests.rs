use cobol_protocol_v154::{base4096, dictionary::CsmDictionary, stream, compress_csm, decompress_csm, CsmError};

#[test]
fn base4096_roundtrip() {
    let data = b"hello world!";
    let encoded = base4096::encode(data);
    let decoded = base4096::decode(&encoded);
    assert_eq!(data, &decoded[..]);
}

#[test]
fn dictionary_insert_lookup() {
    let mut dict = CsmDictionary::new();
    dict.insert(42, b"foobar");
    assert_eq!(dict.lookup(42), Some(&b"foobar"[..]));
}

#[test]
fn dictionary_checksum_verification() {
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"a");
    dict.insert(2, b"b");
    let orig = dict.checksum();
    assert!(dict.verify_checksum());
    dict.insert(3, b"c");
    assert_ne!(dict.checksum(), orig);
}

#[test]
fn stream_header_validation() {
    let dict = CsmDictionary::new();
    let data = b"testdata";
    let compressed = stream::compress_csm_stream(data, &dict).unwrap();
    assert_eq!(&compressed[0..2], b"CS");
    assert_eq!(compressed[2], 0x9A);
}

#[test]
fn crc32_tamper_detection() {
    let dict = CsmDictionary::new();
    let data = b"tamper";
    let mut compressed = stream::compress_csm_stream(data, &dict).unwrap();
    let len = compressed.len();
    compressed[len - 5] ^= 0xFF; // corrupt a byte before CRC
    let res = stream::decompress_csm_stream(&compressed, &dict);
    assert!(matches!(res, Err(CsmError::ChecksumFailed)));
}

#[test]
fn compress_decompress_roundtrip() {
    let dict = CsmDictionary::new();
    let data = b"roundtrip test data";
    let compressed = compress_csm(data, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(data, &decompressed[..]);
}

#[test]
fn test_compress_decompress_binary_zeros() {
    let dict = CsmDictionary::new();
    let input = vec![0u8; 1024];
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn test_compress_decompress_all_same_byte() {
    let dict = CsmDictionary::new();
    let input = vec![0xABu8; 512];
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn test_compress_decompress_alternating() {
    let dict = CsmDictionary::new();
    let mut input = Vec::with_capacity(1024);
    for i in 0..512 {
        input.push(0x00);
        input.push(0xFF);
    }
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn test_compress_empty_dict() {
    let dict = CsmDictionary::new();
    let input = (0..256u8).collect::<Vec<_>>();
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn test_metadata_roundtrip() {
    let dict = CsmDictionary::new();
    let input = (0..128u8).cycle().take(1024).collect::<Vec<_>>();
    let compressed = compress_csm(&input, &dict).unwrap();
    let meta = stream::read_metadata(&compressed).unwrap();
    assert_eq!(meta.magic, [0x43, 0x53]);
    assert_eq!(meta.version, 0x9A);
    assert_eq!(meta.flags & 0x01, 0x00); // dict unused
    assert_eq!(meta.orig_size, input.len() as u32);
}

#[test]
fn test_large_input_chunked() {
    let dict = CsmDictionary::new();
    let input = (0..=255u8).cycle().take(65535).collect::<Vec<_>>();
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

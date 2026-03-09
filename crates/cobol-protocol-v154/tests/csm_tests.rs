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
    let mut dict = CsmDictionary::new();
    let data = b"testdata";
    let compressed = stream::compress_csm_stream(data, &dict).unwrap();
    assert_eq!(&compressed[0..2], b"CS");
    assert_eq!(compressed[2], 0x9A);
}

#[test]
fn crc32_tamper_detection() {
    let mut dict = CsmDictionary::new();
    let data = b"tamper";
    let mut compressed = stream::compress_csm_stream(data, &dict).unwrap();
    let len = compressed.len();
    compressed[len - 5] ^= 0xFF; // corrupt a byte before CRC
    let res = stream::decompress_csm_stream(&compressed, &dict);
    assert!(matches!(res, Err(CsmError::ChecksumFailed)));
}

#[test]
fn compress_decompress_roundtrip() {
    let mut dict = CsmDictionary::new();
    let data = b"roundtrip test data";
    let compressed = compress_csm(data, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(data, &decompressed[..]);
}

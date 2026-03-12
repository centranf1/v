#[test]
fn roundtrip_empty() {
    let dict = CsmDictionary::new();
    let input = vec![];
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn roundtrip_unicode() {
    let dict = CsmDictionary::new();
    let input = "😀漢字テスト🚀".as_bytes().to_vec();
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn roundtrip_random_256() {
    let dict = CsmDictionary::new();
    let input: Vec<u8> = (0..=255u8).collect();
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn roundtrip_max_entry_len() {
    let mut dict = CsmDictionary::new();
    let entry = vec![0xAB; 256];
    dict.insert(1, &entry).unwrap();
    let input = entry.clone();
    let compressed = compress_csm(&input, &dict).unwrap();
    let decompressed = decompress_csm(&compressed, &dict).unwrap();
    assert_eq!(input, decompressed);
}

#[test]
fn dictionary_entry_too_long() {
    let mut dict = CsmDictionary::new();
    let entry = vec![0xCD; 257];
    let res = dict.insert(2, &entry);
    assert!(matches!(res, Err(CsmError::EntryTooLong(_))));
}

#[test]
fn dictionary_full() {
    let mut dict = CsmDictionary::new();
    for i in 0..4096u16 {
        dict.insert(i, &[i as u8]).unwrap();
    }
    let res = dict.insert(4097, b"overflow");
    assert!(matches!(res, Err(CsmError::MaxSymbols)));
}

#[test]
fn dictionary_replace_and_remove() {
    let mut dict = CsmDictionary::new();
    dict.insert(10, b"foo").unwrap();
    dict.insert(10, b"bar").unwrap();
    assert_eq!(dict.lookup(10), Some(&b"bar"[..]));
    dict.remove(10);
    assert_eq!(dict.lookup(10), None);
}

#[test]
fn base4096_padding() {
    let data = b"padme";
    let encoded = base4096::encode(data);
    let mut decoded = base4096::decode(&encoded);
    // Hapus trailing 0 padding jika ada
    while decoded.last() == Some(&0) && decoded.len() > data.len() {
        decoded.pop();
    }
    assert_eq!(data, &decoded[..]);
}

#[test]
fn compress_determinism() {
    let dict = CsmDictionary::new();
    let input = b"deterministic";
    let a = compress_csm(input, &dict).unwrap();
    let b = compress_csm(input, &dict).unwrap();
    assert_eq!(a, b);
}

#[test]
fn decompress_fail_on_corrupt_header() {
    let dict = CsmDictionary::new();
    let mut input = compress_csm(b"fail", &dict).unwrap();
    input[0] = 0x00; // corrupt magic
    let res = decompress_csm(&input, &dict);
    assert!(matches!(res, Err(CsmError::InvalidStream)));
}
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
    dict.insert(42, b"foobar").unwrap();
    assert_eq!(dict.lookup(42), Some(&b"foobar"[..]));
}

#[test]
fn dictionary_checksum_verification() {
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"a").unwrap();
    dict.insert(2, b"b").unwrap();
    let orig = dict.checksum();
    assert!(dict.verify_checksum());
    dict.insert(3, b"c").unwrap();
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
    let input = (0..=255u8).collect::<Vec<_>>();
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

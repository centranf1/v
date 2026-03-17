#![no_main]
use libfuzzer_sys::fuzz_target;
use cobol_protocol_v154::{compress_csm, decompress_csm, dictionary::CsmDictionary};

fuzz_target!(|data: &[u8]| {
    // Create a deterministic dictionary
    let mut dict = CsmDictionary::new();
    
    for i in 0..256.min(data.len()) {
        let entry = &[data[i % data.len()]; 8];
        let _ = dict.insert((i as u16) + 1, entry);
    }

    // First, compress the data
    if let Ok(compressed) = compress_csm(data, &dict) {
        // Decompress should never panic, even if data is modified
        let decompress_result = decompress_csm(&compressed, &dict);
        
        // If decompression succeeds, verify roundtrip
        if let Ok(decompressed) = decompress_result {
            assert_eq!(
                decompressed, data,
                "Decompression roundtrip failed: output mismatch"
            );
        }
        // If decompression fails, it should be a graceful error, not panic
    }
});

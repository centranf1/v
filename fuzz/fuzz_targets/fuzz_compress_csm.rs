#![no_main]
use libfuzzer_sys::fuzz_target;
use cobol_protocol_v154::{compress_csm, dictionary::CsmDictionary};

fuzz_target!(|data: &[u8]| {
    // Create a deterministic dictionary from the input
    let mut dict = CsmDictionary::new();
    
    // Insert up to 256 unique entries derived from input
    for i in 0..256.min(data.len()) {
        let entry = &[data[i % data.len()]; 8];
        let _ = dict.insert((i as u16) + 1, entry);
    }

    // Attempt compression - should never panic
    let result = compress_csm(data, &dict);
    
    // Validate result structure if successful
    if let Ok(compressed) = result {
        // Compressed data should be non-empty if input is non-empty
        if !data.is_empty() {
            assert!(!compressed.is_empty(), "Compression produced empty output for non-empty input");
        }
        
        // Compressed data should not exceed unreasonable size
        // (allow up to 2x input for worst-case scenarios)
        if !data.is_empty() {
            assert!(
                compressed.len() <= data.len() * 2 + 256,
                "Compression output exceeds reasonable bounds: {} bytes for {} bytes input",
                compressed.len(),
                data.len()
            );
        }
    }
});

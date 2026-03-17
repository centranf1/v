#![no_main]
use libfuzzer_sys::fuzz_target;
use cobol_protocol_v154::{compress_csm, decompress_csm, dictionary::CsmDictionary};

fuzz_target!(|data: &[u8]| {
    // Only test on reasonably-sized inputs to avoid timeout
    if data.len() > 1024 * 1024 {
        return;
    }

    // Create dictionary from input
    let mut dict = CsmDictionary::new();
    for i in 0..256.min(data.len()) {
        let entry = &[data[i % data.len()]; 8];
        let _ = dict.insert((i as u16) + 1, entry);
    }

    // Compress
    match compress_csm(data, &dict) {
        Ok(compressed) => {
            // Decompress  
            match decompress_csm(&compressed, &dict) {
                Ok(decompressed) => {
                    // Critical: roundtrip must preserve data
                    assert_eq!(
                        decompressed, data,
                        "CRITICAL: Roundtrip validation failed! Input ≠ Decompress(Compress(input))"
                    );
                }
                Err(e) => {
                    // Decompression should succeed for valid compressed data
                    panic!("Decompression failed for valid compressed data: {}", e);
                }
            }
        }
        Err(_) => {
            // Compression may fail for invalid input - this is acceptable
        }
    }
});

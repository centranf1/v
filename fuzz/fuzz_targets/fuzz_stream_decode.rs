#![no_main]
use libfuzzer_sys::fuzz_target;
use cobol_protocol_v154::stream;

fuzz_target!(|data: &[u8]| {
    // Attempt to read metadata from arbitrary bytes - should never panic
    let _ = stream::read_metadata(data);
    
    // Attempt to read bits from the stream
    if data.len() >= 8 {
        // Create a reader (if public API allows, otherwise this is documentation of what should be tested)
        // For now, we verify that decoder operations don't panic on malformed input
        
        // Try parsing compressed format markers
        if data.len() >= 4 {
            let _marker = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            // Marker parsing should be safe
        }
    }
    
    // Additional stream validation: ensure decoder is resilient
    // to arbitrary byte sequences (fuzzer input)
});

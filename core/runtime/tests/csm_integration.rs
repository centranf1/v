/// Integration tests for CSM compression pipeline

use cnf_compiler::ir::Instruction;
use cnf_runtime::Runtime;
#[test]
fn test_csm_pipeline_dag_scheduler() {
    use cobol_protocol_v154::CsmDictionary;
    use cnf_compiler::ir::Instruction;
    use cnf_runtime::Runtime;

    let mut runtime = Runtime::new();
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"foo");
    dict.insert(2, b"bar");
    runtime.csm_dict = Some(dict.clone());
    runtime.add_buffer("SRC".to_string(), b"hello world".to_vec());

    let instructions = vec![
        Instruction::CompressCsm {
            source: "SRC".to_string(),
            target: "CSM_OUT".to_string(),
        },
        Instruction::DecompressCsm {
            source: "CSM_OUT".to_string(),
            target: "ROUNDTRIP".to_string(),
        },
    ];
    runtime.execute_instructions(&instructions).expect("pipeline execute ok");
    let roundtrip = runtime.get_output("ROUNDTRIP").expect("output exists");
    assert_eq!(roundtrip, b"hello world");
}

#[test]
fn test_csm_template_compression_ratio() {
    use cobol_protocol_v154::CsmDictionary;
    let mut dict = CsmDictionary::new();
    let template = vec![0xAB; 10];
    dict.insert(1, &template);
    let mut runtime = Runtime::new();
    runtime.csm_dict = Some(dict.clone());
    let input = vec![0xAB; 1000];
    runtime.add_buffer("RAW".to_string(), input.clone());
    let instr = Instruction::CompressCsm { source: "RAW".to_string(), target: "CSM".to_string() };
    runtime.execute_instruction(&instr).unwrap();
    let out = runtime.get_output("CSM").unwrap();
    assert!(out.len() < 250, "Dictionary compression should achieve >4x ratio, got {} bytes", out.len());
}

#[test]
fn test_csm_bit_flip_atomic_integrity() {
    use cobol_protocol_v154::CsmDictionary;
    let mut dict = CsmDictionary::new();
    dict.insert(1, &[0xCD; 8]);
    let mut runtime = Runtime::new();
    runtime.csm_dict = Some(dict.clone());
    let input = vec![0xCD; 32];
    runtime.add_buffer("RAW".to_string(), input.clone());
    let instr = Instruction::CompressCsm { source: "RAW".to_string(), target: "CSM".to_string() };
    runtime.execute_instruction(&instr).unwrap();
    let mut out = runtime.get_output("CSM").unwrap();
    // Flip 1 bit in the stream (not in header)
    let idx = 16.min(out.len()-5); // after header
    out[idx] ^= 0x01;
    runtime.add_buffer("FLIPPED".to_string(), out);
    let instr = Instruction::DecompressCsm { source: "FLIPPED".to_string(), target: "FAIL".to_string() };
    let err = runtime.execute_instruction(&instr).unwrap_err();
    assert!(err.to_string().contains("Checksum"), "Expected atomic integrity error");
}

#[test]
fn test_csm_compress_decompress_roundtrip() {
    use cobol_protocol_v154::CsmDictionary;
    use cnf_compiler::ir::Instruction;
    use cnf_runtime::Runtime;
    let mut runtime = Runtime::new();
    let mut dict = CsmDictionary::new();
    dict.insert(1, b"foo");
    dict.insert(2, b"bar");
    runtime.csm_dict = Some(dict.clone());
    runtime.add_buffer("SRC".to_string(), b"hello world".to_vec());

    let compress = Instruction::CompressCsm {
        source: "SRC".to_string(),
        target: "CSM_OUT".to_string(),
    };
    let decompress = Instruction::DecompressCsm {
        source: "CSM_OUT".to_string(),
        target: "ROUNDTRIP".to_string(),
    };

    runtime.execute_instruction(&compress).expect("compress ok");
    runtime.execute_instruction(&decompress).expect("decompress ok");
    let out = runtime.get_output("ROUNDTRIP").unwrap();
    assert_eq!(out, b"hello world");
}

#[test]
fn test_csm_error_when_dict_not_loaded() {
    use cnf_compiler::ir::Instruction;
    use cnf_runtime::Runtime;
    let mut runtime = Runtime::new();
    runtime.add_buffer("SRC".to_string(), b"data".to_vec());
    let compress = Instruction::CompressCsm {
        source: "SRC".to_string(),
        target: "CSM_OUT".to_string(),
    };
    let err = runtime.execute_instruction(&compress).unwrap_err();
    assert!(err.to_string().contains("dictionary not loaded"));
}

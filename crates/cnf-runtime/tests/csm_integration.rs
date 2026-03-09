//! Integration tests for CSM compression pipeline

use cobol_protocol_v154::CsmDictionary;
use cnf_compiler::ir::Instruction;
use cnf_runtime::Runtime;

#[test]
fn test_csm_compress_decompress_roundtrip() {
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
    let mut runtime = Runtime::new();
    runtime.add_buffer("SRC".to_string(), b"data".to_vec());
    let compress = Instruction::CompressCsm {
        source: "SRC".to_string(),
        target: "CSM_OUT".to_string(),
    };
    let err = runtime.execute_instruction(&compress).unwrap_err();
    assert!(err.to_string().contains("dictionary not loaded"));
}

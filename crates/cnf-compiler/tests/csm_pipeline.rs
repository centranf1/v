//! Compiler pipeline test for CSM IR lowering

use cnf_compiler::{compile, ir::Instruction};

#[test]
fn test_compile_csm_statements_to_ir() {
    let source = r#"
        IDENTIFICATION DIVISION.
        PROGRAM-ID. TestCSM.
        ENVIRONMENT DIVISION.
        OS "Linux".
        DATA DIVISION.
        INPUT VIDEO-MP4 AS SRC.
        OUTPUT VIDEO-MP4 AS OUT.
        PROCEDURE DIVISION.
        COMPRESS-CSM SRC INTO OUT.
        DECOMPRESS-CSM OUT INTO SRC.
    "#;
    let ir = compile(source).expect("compile ok");
    assert!(ir.iter().any(|i| matches!(i, Instruction::CompressCsm { source, target } if source == "SRC" && target == "OUT")));
    assert!(ir.iter().any(|i| matches!(i, Instruction::DecompressCsm { source, target } if source == "OUT" && target == "SRC")));
}

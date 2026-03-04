use cnf_compiler::compile;
/// IR Generation Performance Benchmark
///
/// Measures AST → IR lowering overhead.
/// Focuses on semantic analysis and lowering phase.
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn ir_test_program_simple() -> String {
    r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. IRBench.
ENVIRONMENT DIVISION.
    OS "Linux".
DATA DIVISION.
    VAR_1 VIDEO-MP4.
    VAR_2 IMAGE-JPG.
PROCEDURE DIVISION.
    COMPRESS VAR_1 INTO "out.bin".
    VERIFY-INTEGRITY VAR_2.
"#
    .to_string()
}

fn ir_test_program_extended() -> String {
    r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. IRBenchExt.
ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".
DATA DIVISION.
    VAR_1 VIDEO-MP4.
    VAR_2 IMAGE-JPG.
    VAR_3 JSON-OBJECT.
    VAR_4 AUDIO-WAV.
    VAR_5 CSV-TABLE.
    VAR_6 BINARY-BLOB.
    VAR_7 XML-DOCUMENT.
    VAR_8 PARQUET-TABLE.
PROCEDURE DIVISION.
    COMPRESS VAR_1 INTO "out.bin".
    TRANSCODE VAR_1 TO AUDIO-WAV.
    FILTER VAR_5 BY "status = active".
    AGGREGATE VAR_3 WITH "count".
    CONVERT VAR_4 TO JSON-OBJECT.
    MERGE VAR_2 AND VAR_3 INTO VAR_9.
    SPLIT VAR_5 INTO 4 PARTS.
    VALIDATE VAR_6 AGAINST "binary_schema".
    EXTRACT "/root/key" FROM VAR_3.
"#
    .to_string()
}

fn ir_lowering_simple(c: &mut Criterion) {
    c.bench_function("ir_lowering_simple_2_ops", |b| {
        b.iter(|| {
            let program = black_box(ir_test_program_simple());
            compile(&program)
        })
    });
}

fn ir_lowering_extended(c: &mut Criterion) {
    c.bench_function("ir_lowering_extended_8_ops", |b| {
        b.iter(|| {
            let program = black_box(ir_test_program_extended());
            compile(&program)
        })
    });
}

fn ir_lowering_repeated(c: &mut Criterion) {
    c.bench_function("ir_lowering_repeated_100x", |b| {
        b.iter(|| {
            let program = black_box(ir_test_program_simple());
            for _ in 0..100 {
                let _ = compile(&program);
            }
        })
    });
}

criterion_group!(
    benches,
    ir_lowering_simple,
    ir_lowering_extended,
    ir_lowering_repeated
);
criterion_main!(benches);

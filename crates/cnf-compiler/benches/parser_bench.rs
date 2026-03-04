use cnf_compiler::compile;
/// Parser Performance Benchmark
///
/// Measures full parsing pipeline throughput.
/// Exercises lexer + parser + AST construction.
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn valid_program_simple() -> String {
    r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. ParserBench.
ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".
DATA DIVISION.
    VAR_1 VIDEO-MP4.
    VAR_2 IMAGE-JPG.
    VAR_3 FINANCIAL-DECIMAL.
    VAR_4 AUDIO-WAV.
    VAR_5 CSV-TABLE.
PROCEDURE DIVISION.
    COMPRESS VAR_1 INTO "out.bin".
    VERIFY-INTEGRITY VAR_2.
"#
    .to_string()
}

fn valid_program_complex() -> String {
    r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. ParserComplex.
ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".
DATA DIVISION.
    VAR_1 VIDEO-MP4.
    VAR_2 IMAGE-JPG.
    VAR_3 JSON-OBJECT.
    VAR_4 XML-DOCUMENT.
    VAR_5 PARQUET-TABLE.
PROCEDURE DIVISION.
    COMPRESS VAR_1 INTO "out.bin".
    TRANSCODE VAR_1 TO AUDIO-WAV.
    FILTER VAR_5 BY "age > 18".
    AGGREGATE VAR_3 WITH "sum".
    CONVERT VAR_4 TO JSON-OBJECT.
    MERGE VAR_2 AND VAR_3 INTO VAR_6.
"#
    .to_string()
}

fn parser_simple(c: &mut Criterion) {
    c.bench_function("parser_simple_program", |b| {
        b.iter(|| {
            let program = black_box(valid_program_simple());
            compile(&program)
        })
    });
}

fn parser_complex(c: &mut Criterion) {
    c.bench_function("parser_complex_program", |b| {
        b.iter(|| {
            let program = black_box(valid_program_complex());
            compile(&program)
        })
    });
}

fn parser_repeated(c: &mut Criterion) {
    c.bench_function("parser_repeated_10x", |b| {
        b.iter(|| {
            let program = black_box(valid_program_simple());
            for _ in 0..10 {
                let _ = compile(&program);
            }
        })
    });
}

criterion_group!(benches, parser_simple, parser_complex, parser_repeated);
criterion_main!(benches);

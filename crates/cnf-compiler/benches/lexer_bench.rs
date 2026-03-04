use cnf_compiler::lexer::tokenize;
/// Lexer Performance Benchmark
///
/// Measures tokenization throughput across different program sizes.
/// Validates that lexer scales linearly with input size.
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_program(num_var_decls: usize) -> String {
    let mut program = String::from(
        r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. LexerBench.
ENVIRONMENT DIVISION.
    OS "Linux".
DATA DIVISION.
"#,
    );

    // Add variable declarations to scale the program
    for i in 0..num_var_decls {
        program.push_str(&format!("    VAR_{} VIDEO-MP4.\n", i));
    }

    program.push_str(
        r#"PROCEDURE DIVISION.
    COMPRESS VAR_0 INTO "output.bin".
"#,
    );

    program
}

fn lexer_small_program(c: &mut Criterion) {
    c.bench_function("lexer_100_tokens", |b| {
        b.iter(|| {
            let program = black_box(create_program(10));
            tokenize(&program)
        })
    });
}

fn lexer_medium_program(c: &mut Criterion) {
    c.bench_function("lexer_500_tokens", |b| {
        b.iter(|| {
            let program = black_box(create_program(50));
            tokenize(&program)
        })
    });
}

fn lexer_large_program(c: &mut Criterion) {
    c.bench_function("lexer_1000_tokens", |b| {
        b.iter(|| {
            let program = black_box(create_program(100));
            tokenize(&program)
        })
    });
}

criterion_group!(
    benches,
    lexer_small_program,
    lexer_medium_program,
    lexer_large_program
);
criterion_main!(benches);

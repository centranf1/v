use cnf_compiler::compile;
/// Determinism Verification Benchmark
///
/// Critical: Verifies determinism guarantee under high load.
/// Same program → Same IR (byte-for-byte identical) across 1000 compilations.
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn determinism_test_program() -> String {
    r#"IDENTIFICATION DIVISION.
    PROGRAM-ID. DeterminismBench.
ENVIRONMENT DIVISION.
    OS "Linux".
    ARCH "x86_64".
DATA DIVISION.
    VAR_1 VIDEO-MP4.
    VAR_2 IMAGE-JPG.
    VAR_3 JSON-OBJECT.
PROCEDURE DIVISION.
    COMPRESS VAR_1 INTO "out.bin".
    TRANSCODE VAR_1 TO AUDIO-WAV.
    FILTER VAR_2 BY "size > 1000".
    AGGREGATE VAR_3 WITH "sum".
"#
    .to_string()
}

fn determinism_verify_1000x(c: &mut Criterion) {
    c.bench_function("determinism_verify_1000x_compilations", |b| {
        b.iter(|| {
            let program = black_box(determinism_test_program());
            let first_ir = compile(&program).expect("compilation must succeed");
            let first_ir_str = format!("{:?}", first_ir);

            // Verify 999 more times that IR is identical
            for _ in 0..999 {
                let ir = compile(&program).expect("compilation must succeed");
                let ir_str = format!("{:?}", ir);

                // This assertion is part of the benchmark output
                // If this fails, determinism is broken
                if ir_str != first_ir_str {
                    panic!("Determinism violation detected: IR changed after recompilation");
                }
            }
        })
    });
}

fn determinism_stress_10000x(c: &mut Criterion) {
    c.bench_function("determinism_stress_10000x_compilations", |b| {
        b.iter(|| {
            let program = black_box(determinism_test_program());
            let first_ir = compile(&program).expect("compilation must succeed");
            let first_ir_str = format!("{:?}", first_ir);

            // Verify 9999 more times
            for _ in 0..9999 {
                let ir = compile(&program).expect("compilation must succeed");
                let ir_str = format!("{:?}", ir);

                if ir_str != first_ir_str {
                    panic!("Determinism violation under load: IR changed");
                }
            }
        })
    });
}

criterion_group!(
    name = benches;
    config = criterion::Criterion::default().sample_size(10);
    targets = determinism_verify_1000x, determinism_stress_10000x
);
criterion_main!(benches);

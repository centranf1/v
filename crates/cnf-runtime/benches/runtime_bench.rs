use cnf_runtime::Runtime;
/// Runtime Performance Benchmark
///
/// Measures runtime dispatch and buffer management overhead.
/// Tests instruction execution and DAG layer scheduling.
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn runtime_buffer_operations(c: &mut Criterion) {
    c.bench_function("runtime_buffer_add_retrieve", |b| {
        b.iter(|| {
            let mut runtime = Runtime::new();
            let data = black_box(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
            runtime.add_buffer("test".to_string(), data);
            let _result = runtime.get_output("test");
        })
    });
}

fn runtime_multiple_buffers(c: &mut Criterion) {
    c.bench_function("runtime_manage_10_buffers", |b| {
        b.iter(|| {
            let mut runtime = Runtime::new();
            for i in 0..10 {
                let data = black_box(vec![i as u8; 100]);
                runtime.add_buffer(format!("buf_{}", i), data);
            }
            for i in 0..10 {
                let _ = runtime.get_output(&format!("buf_{}", i));
            }
        })
    });
}

fn runtime_execute_dag(c: &mut Criterion) {
    c.bench_function("runtime_execute_dag_scheduling", |b| {
        b.iter(|| {
            let mut runtime = Runtime::new();
            let _result = black_box(runtime.execute());
        })
    });
}

criterion_group!(
    benches,
    runtime_buffer_operations,
    runtime_multiple_buffers,
    runtime_execute_dag
);
criterion_main!(benches);

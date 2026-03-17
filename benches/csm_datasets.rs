#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion, black_box, BenchmarkId};
use cobol_protocol_v154::{compress_csm, decompress_csm, dictionary::CsmDictionary};

/// JSON payload dataset (typical API response)
fn json_payload_dataset() -> Vec<u8> {
    let json = r#"{"users":[
        {"id":1,"name":"Alice","email":"alice@example.com","role":"admin","active":true},
        {"id":2,"name":"Bob","email":"bob@example.com","role":"user","active":true},
        {"id":3,"name":"Charlie","email":"charlie@example.com","role":"user","active":false},
        {"id":4,"name":"Diana","email":"diana@example.com","role":"moderator","active":true},
        {"id":5,"name":"Eve","email":"eve@example.com","role":"admin","active":true}
    ],"timestamp":"2026-03-17T12:34:56Z","count":5,"status":"success"}"#;
    json.repeat(256).into_bytes()
}

/// IoT telemetry dataset (sensor readings)
fn iot_telemetry_dataset() -> Vec<u8> {
    let mut data = Vec::new();
    for i in 0..1000 {
        let reading = format!(
            "sensor_id={:04},timestamp={},temperature={:.1},humidity={:.1},pressure={},status=ok\n",
            i % 100,
            1000000 + i,
            20.0 + (i as f64 % 10.0),
            50.0 + (i as f64 % 20.0),
            1013 + (i % 50)
        );
        data.extend_from_slice(reading.as_bytes());
    }
    data
}

/// Command stream dataset (execution traces)
fn command_stream_dataset() -> Vec<u8> {
    let commands = [
        b"COMPRESS".to_vec(),
        b"DECOMPRESS".to_vec(),
        b"VERIFY_INTEGRITY".to_vec(),
        b"ROTATE_KEY".to_vec(),
        b"CHECKPOINT".to_vec(),
        b"HALT".to_vec(),
    ];
    let mut data = Vec::new();
    for i in 0..5000 {
        let cmd = &commands[i % commands.len()];
        let args = format!("arg1=0x{:08x},arg2=0x{:08x},priority={}\n", i, i * 2, i % 10);
        data.extend_from_slice(cmd);
        data.push(b' ');
        data.extend_from_slice(args.as_bytes());
    }
    data
}

/// Structured logs dataset
fn structured_logs_dataset() -> Vec<u8> {
    let mut data = Vec::new();
    for i in 0..2000 {
        let log_line = format!(
            "[{}] {}::{} - {{level=INFO,module=runtime,event=dispatch_instruction,instruction_id={},duration_us={},success=true}}\n",
            2026_03_17_120000 + i,
            "centra_nf",
            "dispatch",
            i,
            (i * 347) % 10000
        );
        data.extend_from_slice(log_line.as_bytes());
    }
    data
}

/// Binary protobuf-like dataset (simulated)
fn binary_protobuf_dataset() -> Vec<u8> {
    let mut data = Vec::new();
    for i in 0..4096 {
        // Simulate protobuf varint encoding
        let mut val = (i as u32).wrapping_mul(2654435761);
        loop {
            let byte = (val & 0x7F) as u8;
            val >>= 7;
            data.push(byte | if val != 0 { 0x80 } else { 0 });
            if val == 0 {
                break;
            }
        }
    }
    data
}

/// Create a standard dictionary for benchmarking
fn standard_dictionary() -> CsmDictionary {
    let mut dict = CsmDictionary::new();
    // Common patterns
    let patterns = [
        b"timestamp" as &[u8],
        b"status",
        b"success",
        b"error",
        b"module=",
        b"level=",
        b"duration",
        b"user_id",
        b"email",
        b"true",
        b"false",
        b"centra_nf",
        b"dispatch",
        b"compressed",
    ];
    
    for (idx, pattern) in patterns.iter().enumerate() {
        let _ = dict.insert((idx as u16) + 1, pattern);
    }
    dict
}

// ============================================================================
// COMPRESSION BENCHMARKS
// ============================================================================

fn bench_compress_json(c: &mut Criterion) {
    let data = json_payload_dataset();
    let dict = standard_dictionary();
    c.bench_function("compress_json_128kb", |b| {
        b.iter(|| compress_csm(black_box(&data), black_box(&dict)))
    });
}

fn bench_compress_iot(c: &mut Criterion) {
    let data = iot_telemetry_dataset();
    let dict = standard_dictionary();
    c.bench_function("compress_iot_telemetry_185kb", |b| {
        b.iter(|| compress_csm(black_box(&data), black_box(&dict)))
    });
}

fn bench_compress_commands(c: &mut Criterion) {
    let data = command_stream_dataset();
    let dict = standard_dictionary();
    c.bench_function("compress_command_stream_138kb", |b| {
        b.iter(|| compress_csm(black_box(&data), black_box(&dict)))
    });
}

fn bench_compress_logs(c: &mut Criterion) {
    let data = structured_logs_dataset();
    let dict = standard_dictionary();
    c.bench_function("compress_structured_logs_96kb", |b| {
        b.iter(|| compress_csm(black_box(&data), black_box(&dict)))
    });
}

fn bench_compress_binary(c: &mut Criterion) {
    let data = binary_protobuf_dataset();
    let dict = standard_dictionary();
    c.bench_function("compress_binary_proto_16kb", |b| {
        b.iter(|| compress_csm(black_box(&data), black_box(&dict)))
    });
}

// ============================================================================
// ROUNDTRIP BENCHMARKS (Compress → Decompress)
// ============================================================================

fn bench_roundtrip_json(c: &mut Criterion) {
    let data = json_payload_dataset();
    let dict = standard_dictionary();
    if let Ok(compressed) = compress_csm(&data, &dict) {
        c.bench_function("roundtrip_json_128kb", |b| {
            b.iter(|| {
                let out = decompress_csm(black_box(&compressed), black_box(&dict));
                black_box(out)
            })
        });
    }
}

fn bench_roundtrip_iot(c: &mut Criterion) {
    let data = iot_telemetry_dataset();
    let dict = standard_dictionary();
    if let Ok(compressed) = compress_csm(&data, &dict) {
        c.bench_function("roundtrip_iot_telemetry_185kb", |b| {
            b.iter(|| {
                let out = decompress_csm(black_box(&compressed), black_box(&dict));
                black_box(out)
            })
        });
    }
}

fn bench_roundtrip_commands(c: &mut Criterion) {
    let data = command_stream_dataset();
    let dict = standard_dictionary();
    if let Ok(compressed) = compress_csm(&data, &dict) {
        c.bench_function("roundtrip_command_stream_138kb", |b| {
            b.iter(|| {
                let out = decompress_csm(black_box(&compressed), black_box(&dict));
                black_box(out)
            })
        });
    }
}

fn bench_roundtrip_logs(c: &mut Criterion) {
    let data = structured_logs_dataset();
    let dict = standard_dictionary();
    if let Ok(compressed) = compress_csm(&data, &dict) {
        c.bench_function("roundtrip_structured_logs_96kb", |b| {
            b.iter(|| {
                let out = decompress_csm(black_box(&compressed), black_box(&dict));
                black_box(out)
            })
        });
    }
}

// ============================================================================
// COMPARISON: CSM vs RAW (compression ratio)
// ============================================================================

fn bench_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratios");
    group.measurement_time(std::time::Duration::from_secs(5));
    
    let datasets = vec![
        ("json", json_payload_dataset()),
        ("iot", iot_telemetry_dataset()),
        ("commands", command_stream_dataset()),
        ("logs", structured_logs_dataset()),
        ("binary", binary_protobuf_dataset()),
    ];
    
    let dict = standard_dictionary();
    
    for (name, data) in datasets {
        if let Ok(compressed) = compress_csm(&data, &dict) {
            let ratio = compressed.len() as f64 / data.len() as f64;
            group.bench_with_input(
                BenchmarkId::from_parameter(name),
                &ratio,
                |b, ratio| {
                    b.iter(|| {
                        black_box(ratio);
                    })
                },
            );
        }
    }
    group.finish();
}

// ============================================================================
// CRITERION CONFIGURATION
// ============================================================================

criterion_group!(
    name = compression_benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_compress_json,
             bench_compress_iot,
             bench_compress_commands,
             bench_compress_logs,
             bench_compress_binary,
             bench_roundtrip_json,
             bench_roundtrip_iot,
             bench_roundtrip_commands,
             bench_roundtrip_logs,
             bench_compression_ratio
);

criterion_main!(compression_benches);

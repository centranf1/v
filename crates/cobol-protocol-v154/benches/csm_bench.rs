use criterion::{criterion_group, criterion_main, Criterion, black_box};
use cobol_protocol_v154::{base4096, dictionary::CsmDictionary, compress_csm, decompress_csm, stream};

fn bench_base4096_encode(c: &mut Criterion) {
    let data = vec![0xAA; 4096];
    c.bench_function("base4096_encode_4k", |b| b.iter(|| base4096::encode(black_box(&data))));
}

fn bench_dictionary_lookup(c: &mut Criterion) {
    let mut dict = CsmDictionary::new();
    for i in 0..1024u16 {
        dict.insert(i, &[0xBB; 8]);
    }
    c.bench_function("dict_lookup_1k", |b| b.iter(|| dict.lookup(black_box(512))));
}

fn bench_compress_repetitive(c: &mut Criterion) {
    let pattern = b"CENTRA-NF ";
    let mut dict = CsmDictionary::new();
    dict.insert(1, pattern);
    let input = pattern.repeat(4096 / pattern.len());
    c.bench_function("compress_repetitive", |b| {
        b.iter(|| compress_csm(black_box(&input), black_box(&dict)))
    });
}

fn bench_compress_random(c: &mut Criterion) {
    let mut state = 0xDEADBEEFCAFEBABEu64;
    let mut input = vec![0u8; 4096];
    for b in &mut input {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = (state >> 32) as u8;
    }
    let dict = CsmDictionary::new();
    c.bench_function("compress_random", |b| {
        b.iter(|| compress_csm(black_box(&input), black_box(&dict)))
    });
}

fn bench_decompress_roundtrip(c: &mut Criterion) {
    let mut state = 0x1234567890ABCDEFu64;
    let mut input = vec![0u8; 4096];
    for b in &mut input {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = (state >> 32) as u8;
    }
    let dict = CsmDictionary::new();
    let compressed = compress_csm(&input, &dict).unwrap();
    c.bench_function("decompress_roundtrip", |b| {
        b.iter(|| {
            let out = decompress_csm(black_box(&compressed), black_box(&dict)).unwrap();
            black_box(out)
        })
    });
}

fn bench_read_metadata(c: &mut Criterion) {
    let input = vec![0x42; 4096];
    let dict = CsmDictionary::new();
    let compressed = compress_csm(&input, &dict).unwrap();
    c.bench_function("read_metadata", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _ = stream::read_metadata(black_box(&compressed));
            }
        })
    });
}

fn bench_compress_random_large_dict(c: &mut Criterion) {
    let mut input = vec![0u8; 1024 * 1024];
    for i in 0..input.len() {
        input[i] = (i % 256) as u8;
    }
    let mut dict = CsmDictionary::new();
    for i in 1..=1000u16 {
        let entry = vec![(i % 256) as u8; 8];
        dict.insert(i, &entry).unwrap();
    }
    c.bench_function("compress_random_large_dict", |b| {
        b.iter(|| {
            let out = compress_csm(black_box(&input), black_box(&dict));
            black_box(out).unwrap();
        })
    });
}

criterion_group!(benches,
    bench_base4096_encode,
    bench_dictionary_lookup,
    bench_compress_repetitive,
    bench_compress_random,
    bench_compress_random_large_dict,
    bench_decompress_roundtrip,
    bench_read_metadata,
);
criterion_main!(benches);

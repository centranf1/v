use criterion::{criterion_group, criterion_main, Criterion};
use cobol_protocol_v154::{base4096, dictionary::CsmDictionary};

fn bench_base4096_encode(c: &mut Criterion) {
    let data = vec![0xAA; 4096];
    c.bench_function("base4096_encode_4k", |b| b.iter(|| base4096::encode(&data)));
}

fn bench_dictionary_lookup(c: &mut Criterion) {
    let mut dict = CsmDictionary::new();
    for i in 0..1024u16 {
        dict.insert(i, &[0xBB; 8]);
    }
    c.bench_function("dict_lookup_1k", |b| b.iter(|| dict.lookup(512)));
}

criterion_group!(benches, bench_base4096_encode, bench_dictionary_lookup);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dissasemble_mov::dissasemble;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("listing 37", |b| {
        b.iter(|| {
            dissasemble(black_box(include_bytes!(
                "../tests/cases/listing_0037_single_register_mov"
            )))
        })
    });
    c.bench_function("listing 38", |b| {
        b.iter(|| {
            dissasemble(black_box(include_bytes!(
                "../tests/cases/listing_0038_many_register_mov"
            )))
        })
    });
    c.bench_function("listing 39", |b| {
        b.iter(|| {
            dissasemble(black_box(include_bytes!(
                "../tests/cases/listing_0039_more_movs"
            )))
        })
    });
    c.bench_function("listing 40", |b| {
        b.iter(|| {
            dissasemble(black_box(include_bytes!(
                "../tests/cases/listing_0040_challenge_movs"
            )))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

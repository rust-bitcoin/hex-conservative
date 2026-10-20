// SPDX-License-Identifier: CC0-1.0

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hex_conservative::buf_encoder::BufEncoder;
use hex_conservative::Case;

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("buf_encoder");
    group.warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    let bytes = [0xab; 512];
    for len in [0, 1, 32, 511, 512] {
        let input = &bytes[..len];
        group.bench_function(BenchmarkId::new("fresh", len), |b| {
            b.iter(|| {
                let mut encoder = BufEncoder::<1024>::new(Case::Lower);
                encoder.put_bytes(black_box(input));
                black_box(encoder.as_str());
            });
        });
        let mut encoder = BufEncoder::<1024>::new(Case::Lower);
        group.bench_function(BenchmarkId::new("reuse", len), |b| {
            b.iter(|| {
                encoder.clear();
                encoder.put_bytes(black_box(input));
                black_box(encoder.as_str());
            });
        });
        encoder.clear();
        encoder.put_bytes(input);
        group.bench_function(BenchmarkId::new("clone", len), |b| {
            b.iter(|| black_box(black_box(&encoder).clone()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_encode);
criterion_main!(benches);

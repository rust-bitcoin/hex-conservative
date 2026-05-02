// SPDX-License-Identifier: CC0-1.0

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hex_conservative::{decode_to_array, decode_to_vec};

fn bench_decode_to_array(c: &mut Criterion) {
    let mut g = c.benchmark_group("decode_to_array");
    g.warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));

    // 4 bytes / 8 hex chars
    let hex_8 = "deadbeef";
    g.bench_function(BenchmarkId::new("bytes", 4), |b| {
        b.iter(|| decode_to_array::<4>(black_box(hex_8)).unwrap());
    });

    // 32 bytes / 64 hex chars
    let hex_64 = "deadbeef".repeat(8);
    g.bench_function(BenchmarkId::new("bytes", 32), |b| {
        b.iter(|| decode_to_array::<32>(black_box(hex_64.as_str())).unwrap());
    });

    // 256 bytes / 512 hex chars
    let hex_512 = "ab".repeat(256);
    g.bench_function(BenchmarkId::new("bytes", 256), |b| {
        b.iter(|| decode_to_array::<256>(black_box(hex_512.as_str())).unwrap());
    });

    g.finish();
}

fn bench_decode_to_vec(c: &mut Criterion) {
    let mut g = c.benchmark_group("decode_to_vec");
    g.warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));

    // 4 bytes / 8 hex chars
    let hex_8 = "deadbeef";
    g.bench_function(BenchmarkId::new("bytes", 4), |b| {
        b.iter(|| decode_to_vec(black_box(hex_8)).unwrap());
    });

    // 32 bytes / 64 hex chars
    let hex_64 = "deadbeef".repeat(8);
    g.bench_function(BenchmarkId::new("bytes", 32), |b| {
        b.iter(|| decode_to_vec(black_box(hex_64.as_str())).unwrap());
    });

    // 256 bytes / 512 hex chars
    let hex_512 = "ab".repeat(256);
    g.bench_function(BenchmarkId::new("bytes", 256), |b| {
        b.iter(|| decode_to_vec(black_box(hex_512.as_str())).unwrap());
    });

    g.finish();
}

criterion_group!(benches, bench_decode_to_array, bench_decode_to_vec);
criterion_main!(benches);

// SPDX-License-Identifier: CC0-1.0

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hex_conservative::{BytesToHexIter, Case, Char};

fn nth_positions(size: usize) -> [usize; 5] {
    [size / 8, size / 4, size / 2, size * 3 / 4, size - 1]
}

fn slow_nth(
    iter: &mut BytesToHexIter<core::slice::Iter<'_, u8>>,
    n: usize,
) -> Option<[Char; 2]> {
    for _ in 0..n {
        iter.next()?;
    }
    iter.next()
}

fn slow_nth_back(
    iter: &mut BytesToHexIter<core::slice::Iter<'_, u8>>,
    n: usize,
) -> Option<[Char; 2]> {
    for _ in 0..n {
        iter.next_back()?;
    }
    iter.next_back()
}

fn bench_bytes_to_hex_nth(c: &mut Criterion) {
    let mut g = c.benchmark_group("bytes_to_hex_iter_nth");
    g.warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));

    for &size in &[128usize, 4096] {
        let bytes = vec![0x5a; size];
        let positions = nth_positions(size);

        g.bench_function(BenchmarkId::new("optimized", size), |b| {
            b.iter(|| {
                let mut sum = 0u32;
                for &n in &positions {
                    let mut iter = BytesToHexIter::new(black_box(bytes.as_slice()).iter(), Case::Lower);
                    let [hi, lo] = iter.nth(black_box(n)).unwrap();
                    sum += u32::from(u8::from(hi)) + u32::from(u8::from(lo));
                }
                black_box(sum)
            });
        });

        g.bench_function(BenchmarkId::new("linear_baseline", size), |b| {
            b.iter(|| {
                let mut sum = 0u32;
                for &n in &positions {
                    let mut iter = BytesToHexIter::new(black_box(bytes.as_slice()).iter(), Case::Lower);
                    let [hi, lo] = slow_nth(&mut iter, black_box(n)).unwrap();
                    sum += u32::from(u8::from(hi)) + u32::from(u8::from(lo));
                }
                black_box(sum)
            });
        });
    }

    g.finish();
}

fn bench_bytes_to_hex_nth_back(c: &mut Criterion) {
    let mut g = c.benchmark_group("bytes_to_hex_iter_nth_back");
    g.warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));

    for &size in &[128usize, 4096] {
        let bytes = vec![0x5a; size];
        let positions = nth_positions(size);

        g.bench_function(BenchmarkId::new("optimized", size), |b| {
            b.iter(|| {
                let mut sum = 0u32;
                for &n in &positions {
                    let mut iter = BytesToHexIter::new(black_box(bytes.as_slice()).iter(), Case::Lower);
                    let [hi, lo] = iter.nth_back(black_box(n)).unwrap();
                    sum += u32::from(u8::from(hi)) + u32::from(u8::from(lo));
                }
                black_box(sum)
            });
        });

        g.bench_function(BenchmarkId::new("linear_baseline", size), |b| {
            b.iter(|| {
                let mut sum = 0u32;
                for &n in &positions {
                    let mut iter = BytesToHexIter::new(black_box(bytes.as_slice()).iter(), Case::Lower);
                    let [hi, lo] = slow_nth_back(&mut iter, black_box(n)).unwrap();
                    sum += u32::from(u8::from(hi)) + u32::from(u8::from(lo));
                }
                black_box(sum)
            });
        });
    }

    g.finish();
}

criterion_group!(benches, bench_bytes_to_hex_nth, bench_bytes_to_hex_nth_back);
criterion_main!(benches);

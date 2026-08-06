//! Criterion benchmarks for deterministic randomization and replay.

mod random_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use random_support::{
    BENCH_REPLAY, BENCH_SEED, RandomTransaction, RandomTransactionSequence, configure_group,
    throughput_elements,
};
use vvm_core::{Bits, RandomContext, Randomize, SignedBits};

/// Generates deterministic `Bits` from the VVM random stream.
fn random_bits<const N: usize>(random: &mut RandomContext) -> Bits<N> {
    let word_count = N.div_ceil(32);
    let words = (0..word_count)
        .map(|_| random.next_u32())
        .collect::<Vec<_>>();

    Bits::<N>::from_words_le(words).unwrap_or_default()
}

/// Generates deterministic `SignedBits` from the VVM random stream.
fn random_signed_bits<const N: usize>(random: &mut RandomContext) -> SignedBits<N> {
    let word_count = N.div_ceil(32);
    let words = (0..word_count)
        .map(|_| random.next_u32())
        .collect::<Vec<_>>();

    SignedBits::<N>::from_words_le(words).unwrap_or_default()
}

/// Benchmarks scalar and packed random generation.
fn random_generation_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("random/generation");
    configure_group(&mut group);

    group.bench_function("scalar/bool", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(bool::randomize(black_box(&mut random))));
    });

    group.bench_function("scalar/u32", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(u32::randomize(black_box(&mut random))));
    });

    group.bench_function("scalar/i64", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(i64::randomize(black_box(&mut random))));
    });

    group.bench_function("packed/bits/128", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(random_bits::<128>(black_box(&mut random))));
    });

    group.bench_function("packed/signed/129", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(random_signed_bits::<129>(black_box(&mut random))));
    });

    group.bench_function("transaction/structured", |bench| {
        let mut random = RandomContext::new(BENCH_SEED);
        bench.iter(|| black_box(RandomTransaction::randomize(black_box(&mut random))));
    });

    group.finish();
}

/// Benchmarks replayable deterministic sequence generation.
fn replay_sequence_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("random/replay");
    configure_group(&mut group);

    for length in [16_usize, 256, 4096] {
        throughput_elements(&mut group, u64::try_from(length).unwrap_or_default());

        group.bench_with_input(
            BenchmarkId::new("sequence", length),
            &length,
            |bench, &len| {
                bench.iter(|| {
                    let values = RandomTransactionSequence::new(BENCH_REPLAY, len)
                        .into_iter()
                        .collect::<Vec<_>>();
                    black_box(values)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("same-seed", length),
            &length,
            |bench, &len| {
                bench.iter(|| {
                    let left = RandomTransactionSequence::new(BENCH_REPLAY, len)
                        .into_iter()
                        .collect::<Vec<_>>();
                    let right = RandomTransactionSequence::new(BENCH_REPLAY, len)
                        .into_iter()
                        .collect::<Vec<_>>();
                    black_box(left == right)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    random_benches,
    random_generation_benches,
    replay_sequence_benches
);
criterion_main!(random_benches);

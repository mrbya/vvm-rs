//! Criterion benchmarks for subprocess-oriented `cargo-vvm` workflows.

mod orchestration_support;

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use orchestration_support::{PreparedCommand, configure_group};

/// Benchmarks representative `cargo-vvm` command paths.
fn orchestration_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cargo-vvm/subprocess");
    configure_group(&mut group);
    let Ok(binary) = PreparedCommand::build_binary() else {
        return;
    };

    group.bench_function("help", |bench| {
        bench.iter_batched(
            || PreparedCommand::new(binary.clone()).ok(),
            |command| {
                black_box(
                    command
                        .as_ref()
                        .is_some_and(|prepared| prepared.help().is_ok()),
                )
            },
            BatchSize::PerIteration,
        );
    });

    group.bench_function("coverage/counter", |bench| {
        bench.iter_batched(
            || PreparedCommand::new(binary.clone()).ok(),
            |command| {
                black_box(
                    command
                        .as_ref()
                        .is_some_and(|prepared| prepared.counter_coverage().is_ok()),
                )
            },
            BatchSize::PerIteration,
        );
    });

    group.finish();
}

criterion_group!(orchestration_benchmark_group, orchestration_benches);
criterion_main!(orchestration_benchmark_group);

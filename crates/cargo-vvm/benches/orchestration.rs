//! Criterion benchmarks for subprocess-oriented `cargo-vvm` workflows.

mod orchestration_support;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use orchestration_support::{PreparedCommand, configure_group};

/// Benchmarks representative `cargo-vvm` command paths.
fn orchestration_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("cargo-vvm/subprocess");
    configure_group(&mut group);
    let binary = PreparedCommand::build_binary().unwrap_or_else(|_| unreachable!());

    group.bench_function("help", |bench| {
        bench.iter_batched(
            || PreparedCommand::new(binary.clone()).unwrap_or_else(|_| unreachable!()),
            |command| command.help(),
            BatchSize::PerIteration,
        );
    });

    group.bench_function("coverage/counter", |bench| {
        bench.iter_batched(
            || PreparedCommand::new(binary.clone()).unwrap_or_else(|_| unreachable!()),
            |command| command.counter_coverage(),
            BatchSize::PerIteration,
        );
    });

    group.finish();
}

criterion_group!(orchestration_benchmark_group, orchestration_benches);
criterion_main!(orchestration_benchmark_group);

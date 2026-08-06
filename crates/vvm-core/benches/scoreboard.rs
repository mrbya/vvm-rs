//! Criterion benchmarks for scoreboards and deterministic mismatch paths.

mod scoreboard_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use scoreboard_support::{StructuredValue, configure_group, throughput_elements};
use vvm_core::{ExactScoreboard, Scoreboard};

/// Benchmarks exact scoreboard comparison costs.
fn scoreboard_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("scoreboard/exact");
    configure_group(&mut group);

    throughput_elements(&mut group, 1);

    group.bench_function("scalar/success", |bench| {
        bench.iter(|| {
            let mut scoreboard = ExactScoreboard;
            let passed = scoreboard.check(black_box(7_u32), black_box(7_u32)).is_ok();
            black_box(passed)
        });
    });

    group.bench_function("structured/success", |bench| {
        let expected = StructuredValue::new(0x55);
        let observed = StructuredValue::new(0x55);

        bench.iter(|| {
            let mut scoreboard = ExactScoreboard;
            let passed = scoreboard
                .check(black_box(expected.clone()), black_box(observed.clone()))
                .is_ok();
            black_box(passed)
        });
    });

    for (name, observed) in [
        (
            "structured/early-mismatch",
            StructuredValue::new(0x55).early_mismatch(),
        ),
        (
            "structured/late-mismatch",
            StructuredValue::new(0x55).late_mismatch(),
        ),
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &observed,
            |bench, item| {
                let expected = StructuredValue::new(0x55);

                bench.iter(|| {
                    let mut scoreboard = ExactScoreboard;
                    let mismatch =
                        scoreboard.check(black_box(expected.clone()), black_box(item.clone()));
                    black_box(mismatch.is_err())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(scoreboard_benchmark_group, scoreboard_benches);
criterion_main!(scoreboard_benchmark_group);

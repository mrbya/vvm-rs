//! Criterion benchmarks for subprocess-oriented `vvm-build` workflows.

mod build_support;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use build_support::{PreparedFixture, configure_group};

/// Benchmarks isolated clean, warm, and incremental `vvm-build` workflows.
fn build_workflow_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vvm-build/subprocess");
    configure_group(&mut group);

    group.bench_function("clean/build-consumer", |bench| {
        bench.iter_batched(
            || PreparedFixture::build_consumer().unwrap_or_else(|_| unreachable!()),
            |fixture| fixture.cargo_test_no_run(),
            BatchSize::PerIteration,
        );
    });

    group.bench_function("warm-noop/build-consumer", |bench| {
        bench.iter_batched(
            || {
                let fixture = PreparedFixture::build_consumer().unwrap_or_else(|_| unreachable!());
                fixture.cargo_test_no_run().unwrap_or_else(|_| unreachable!());
                fixture
            },
            |fixture| fixture.cargo_test_no_run(),
            BatchSize::PerIteration,
        );
    });

    for (name, relative_path, marker) in [
        ("rust-incremental/build-consumer", "src/lib.rs", "// bench rust incremental"),
        ("hdl-incremental/build-consumer", "rtl/counter.sv", "// bench hdl incremental"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(name), &relative_path, |bench, path| {
            bench.iter_batched(
                || {
                    let fixture = PreparedFixture::build_consumer().unwrap_or_else(|_| unreachable!());
                    fixture.cargo_test_no_run().unwrap_or_else(|_| unreachable!());
                    fixture.append_line(path, marker).unwrap_or_else(|_| unreachable!());
                    fixture
                },
                |fixture| fixture.cargo_test_no_run(),
                BatchSize::PerIteration,
            );
        });
    }

    group.bench_function("mixed-port/native-wide-transform", |bench| {
        bench.iter_batched(
            || {
                PreparedFixture::from_fixture("native-wide-transform", "native-wide-transform")
                    .unwrap_or_else(|_| unreachable!())
            },
            |fixture| fixture.cargo_test_no_run(),
            BatchSize::PerIteration,
        );
    });

    group.finish();
}

criterion_group!(build_workflow_benchmark_group, build_workflow_benches);
criterion_main!(build_workflow_benchmark_group);

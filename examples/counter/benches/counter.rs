//! Criterion benchmarks for the native counter example.

/// Shared native counter benchmark configuration.
mod support {
    use std::time::Duration;

    use criterion::measurement::WallTime;
    use criterion::{BenchmarkGroup, Throughput};

    /// Applies the native benchmark sampling policy.
    pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
        group.sample_size(20);
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(5));
    }

    /// Reports cycle throughput for counter-native workloads.
    pub fn throughput_cycles(group: &mut BenchmarkGroup<'_, WallTime>, cycles: u64) {
        group.throughput(Throughput::Elements(cycles));
    }
}

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;
use vvm::testbench::TestResult;
use vvm_example_counter::benchmark::{
    BENCH_CYCLES, run_raw_cycles, run_smoke_testbench, run_testbench, run_testbench_with_coverage,
};

/// Benchmarks native counter cycle and verification overhead.
fn counter_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("native/counter");
    support::configure_group(&mut group);
    support::throughput_cycles(&mut group, BENCH_CYCLES);

    group.bench_function("raw-cycle", |bench| {
        bench.iter(|| black_box(run_raw_cycles(BENCH_CYCLES)));
    });

    group.bench_function("testbench-cycle", |bench| {
        bench.iter(|| {
            let result = run_testbench(None);
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("testbench-cycle/smoke", |bench| {
        bench.iter(|| {
            let result = run_smoke_testbench();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("testbench-cycle/traced", |bench| {
        bench.iter(|| {
            tempdir().map_or_else(
                |_| black_box(false),
                |directory| {
                    let trace_path = directory.path().join("counter.vcd");
                    let result = run_testbench(Some(&trace_path));
                    black_box(result.as_ref().is_ok_and(TestResult::passed))
                },
            )
        });
    });

    group.bench_function("testbench-cycle/covered", |bench| {
        bench.iter(|| {
            let result = run_testbench_with_coverage(None);
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.finish();
}

criterion_group!(counter_benchmark_group, counter_benches);
criterion_main!(counter_benchmark_group);

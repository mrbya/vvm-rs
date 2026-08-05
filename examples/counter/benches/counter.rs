//! Criterion benchmarks for the native counter example.

mod support {
    use std::time::Duration;

    use criterion::{BenchmarkGroup, Throughput, measurement::WallTime};

    pub fn configure_group(group: &mut BenchmarkGroup<'_, WallTime>) {
        group.sample_size(20);
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(5));
    }

    pub fn throughput_cycles(group: &mut BenchmarkGroup<'_, WallTime>, cycles: u64) {
        group.throughput(Throughput::Elements(cycles));
    }
}

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;
use vvm_example_counter::benchmark::{
    BENCH_CYCLES, run_raw_cycles, run_smoke_testbench, run_testbench, run_testbench_with_coverage,
};

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
            assert!(result.as_ref().is_ok_and(|run| run.passed()));
            black_box(result)
        });
    });

    group.bench_function("testbench-cycle/smoke", |bench| {
        bench.iter(|| {
            let result = run_smoke_testbench();
            assert!(result.as_ref().is_ok_and(|run| run.passed()));
            black_box(result)
        });
    });

    group.bench_function("testbench-cycle/traced", |bench| {
        bench.iter(|| {
            let directory = tempdir().unwrap_or_else(|_| unreachable!());
            let trace_path = directory.path().join("counter.vcd");
            let result = run_testbench(Some(&trace_path));
            assert!(result.as_ref().is_ok_and(|run| run.passed()));
            black_box(result)
        });
    });

    group.bench_function("testbench-cycle/covered", |bench| {
        bench.iter(|| {
            let result = run_testbench_with_coverage(None);
            assert!(result.as_ref().is_ok_and(|run| run.passed()));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(counter_benchmark_group, counter_benches);
criterion_main!(counter_benchmark_group);

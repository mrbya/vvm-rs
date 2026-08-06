//! Criterion benchmarks for the native synchronous FIFO example.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use vvm::testbench::TestResult;
use vvm_example_sync_fifo::benchmark::{
    BENCH_CYCLES, run_pop_only, run_push_only, run_simultaneous, run_testbench,
    run_testbench_with_coverage,
};

/// Benchmarks native synchronous FIFO transaction throughput.
fn sync_fifo_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("native/sync-fifo");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Elements(BENCH_CYCLES));

    group.bench_function("transactions", |bench| {
        bench.iter(|| {
            let result = run_testbench();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("transactions/push-only", |bench| {
        bench.iter(|| {
            let result = run_push_only();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("transactions/pop-only", |bench| {
        bench.iter(|| {
            let result = run_pop_only();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("transactions/simultaneous", |bench| {
        bench.iter(|| {
            let result = run_simultaneous();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.bench_function("transactions/covered", |bench| {
        bench.iter(|| {
            let result = run_testbench_with_coverage();
            black_box(result.as_ref().is_ok_and(TestResult::passed))
        });
    });

    group.finish();
}

criterion_group!(sync_fifo_benchmark_group, sync_fifo_benches);
criterion_main!(sync_fifo_benchmark_group);

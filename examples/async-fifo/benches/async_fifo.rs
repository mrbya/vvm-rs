//! Criterion benchmarks for the native asynchronous FIFO example.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use vvm_example_async_fifo::benchmark::{
    BENCH_ROUNDS, run_coincident_edges, run_fill_and_drain, run_randomized,
    run_read_faster_than_write, run_write_faster_than_read,
};

/// Benchmarks native asynchronous FIFO throughput under fixed clock relationships.
fn async_fifo_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("native/async-fifo");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Elements(u64::from(BENCH_ROUNDS)));

    group.bench_function("transactions", |bench| {
        bench.iter(|| {
            let result = run_randomized();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("transactions/fill-drain", |bench| {
        bench.iter(|| {
            let result = run_fill_and_drain();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("transactions/coincident", |bench| {
        bench.iter(|| {
            let result = run_coincident_edges();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("transactions/write-faster", |bench| {
        bench.iter(|| {
            let result = run_write_faster_than_read();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("transactions/read-faster", |bench| {
        bench.iter(|| {
            let result = run_read_faster_than_write();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(async_fifo_benchmark_group, async_fifo_benches);
criterion_main!(async_fifo_benchmark_group);

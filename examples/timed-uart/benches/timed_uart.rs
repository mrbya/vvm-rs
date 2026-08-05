//! Criterion benchmarks for the native timed-UART example.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use vvm_example_timed_uart::benchmark::{BENCH_FRAMES, run_randomized, run_randomized_with_coverage};

fn timed_uart_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("native/timed-uart");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Elements(u64::from(BENCH_FRAMES)));

    group.bench_function("frames", |bench| {
        bench.iter(|| {
            let result = run_randomized();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.bench_function("frames/covered", |bench| {
        bench.iter(|| {
            let result = run_randomized_with_coverage();
            assert!(result.is_ok());
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(timed_uart_benchmark_group, timed_uart_benches);
criterion_main!(timed_uart_benchmark_group);

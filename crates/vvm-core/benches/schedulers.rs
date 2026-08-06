//! Criterion benchmarks for testbench and scheduler execution.

mod scheduler_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use scheduler_support::{
    MockClock, MockDut, MockModel, MockObservation, MockStimulus, configure_expensive_group,
    configure_group, dense_event_ticks, run_mock_testbench, run_multiclock_testbench,
    run_timing_scheduler, sparse_event_ticks, throughput_cycles, throughput_events,
};
use vvm_core::{ExactScoreboard, Testbench};

/// Benchmarks deterministic mock-DUT testbench execution.
fn testbench_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("testbench/mock-dut");
    configure_expensive_group(&mut group);

    for cycles in [64_usize, 256] {
        throughput_cycles(&mut group, u64::try_from(cycles).unwrap_or_default());

        group.bench_with_input(
            BenchmarkId::new("model-scoreboard", cycles),
            &cycles,
            |bench, &count| {
                bench.iter(|| run_mock_testbench(count));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("observer", cycles),
            &cycles,
            |bench, &count| {
                let sequence = MockStimulus::sequence(count);

                bench.iter(|| {
                    Testbench::new(MockDut::default())
                        .with_sequence(sequence.clone())
                        .with_reference_model(MockModel::default())
                        .with_scoreboard(ExactScoreboard)
                        .with_clock(MockClock)
                        .run_with_observer::<MockObservation, _>(|cycle| {
                            black_box(cycle.cycle());
                        })
                });
            },
        );
    }

    group.finish();
}

/// Benchmarks multi-clock scheduler event processing.
fn clock_scheduler_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("scheduler/multi-clock");
    configure_group(&mut group);

    for (name, clock_count) in [("1-clock", 1_usize), ("2-clocks", 2), ("4-clocks", 4)] {
        let cycles = 128_usize;
        throughput_events(&mut group, u64::try_from(cycles).unwrap_or_default());

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &clock_count,
            |bench, &count| {
                bench.iter(|| run_multiclock_testbench(cycles, count));
            },
        );
    }

    group.finish();
}

/// Benchmarks timed delayed-event scheduling.
fn timing_scheduler_benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("scheduler/timing");
    configure_group(&mut group);

    for (name, events) in [
        ("dense-small", dense_event_ticks(64)),
        ("dense-large", dense_event_ticks(512)),
        ("sparse-small", sparse_event_ticks(64)),
        ("sparse-large", sparse_event_ticks(512)),
    ] {
        throughput_events(&mut group, u64::try_from(events.len()).unwrap_or_default());

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &events,
            |bench, ticks| {
                bench.iter(|| run_timing_scheduler(ticks));
            },
        );
    }

    group.finish();
}

criterion_group!(
    scheduler_benches,
    testbench_benches,
    clock_scheduler_benches,
    timing_scheduler_benches
);
criterion_main!(scheduler_benches);

//! Hidden native benchmark harness for the counter example.

use std::path::Path;

use vvm::coverage::{CoverageGroup, CoverageModel};
use vvm::random::{ReplayToken, Seed};
use vvm::testbench::{ExactScoreboard, Testbench};

use crate::counter::Counter;
use crate::coverage::CounterCoverage;
use crate::verification::{
    CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult,
    RandomCounterSequence, Result, counter_sequence,
};

/// Number of deterministic cycles used by the native benchmark workloads.
pub const BENCH_CYCLES: u64 = 2_000;
/// Stable benchmark replay stream.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x72d7_5a12_993e_8411));

/// Runs a raw generated-DUT cycle loop without testbench overhead.
pub fn run_raw_cycles(cycles: u64) -> Result<u64> {
    let mut dut = Counter::new()?;
    dut.set_clk(false)?;
    dut.set_reset_n(false)?;
    dut.set_enable(false)?;
    dut.eval()?;

    dut.set_clk(true)?;
    dut.eval()?;
    dut.set_clk(false)?;
    dut.eval()?;

    dut.set_reset_n(true)?;
    dut.set_enable(true)?;

    for _ in 0..cycles {
        dut.set_clk(true)?;
        dut.eval()?;
        dut.set_clk(false)?;
        dut.eval()?;
    }

    let count = u64::from(dut.count()?);
    dut.finish()?;

    Ok(count)
}

/// Runs the counter through the normal VVM testbench without coverage.
pub fn run_testbench(trace_path: Option<&Path>) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    if let Some(path) = trace_path {
        dut.open_trace(path)?;
    }

    let sequence = RandomCounterSequence::new(BENCH_REPLAY, BENCH_CYCLES);

    Ok(Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>())
}

/// Runs the deterministic directed counter scenario.
pub fn run_smoke_testbench() -> Result<CounterTestResult> {
    let dut = Counter::new()?;

    Ok(Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>())
}

/// Runs the counter through the normal VVM testbench while sampling coverage.
pub fn run_testbench_with_coverage(trace_path: Option<&Path>) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    if let Some(path) = trace_path {
        dut.open_trace(path)?;
    }

    let sequence = RandomCounterSequence::new(BENCH_REPLAY, BENCH_CYCLES);
    let mut coverage = CounterCoverage::new("dut.counter.bench")?;

    let result = Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run_with_observer::<CounterObservation, _>(|cycle| {
            let sampled = coverage.sample(cycle);
            assert!(
                sampled.is_ok(),
                "counter coverage sampling must succeed during benches"
            );
        });

    let validated = coverage.validate();
    assert!(
        validated.is_ok(),
        "counter coverage definition must stay valid during benches"
    );

    Ok(result)
}

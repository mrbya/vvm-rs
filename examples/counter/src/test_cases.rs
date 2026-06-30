use vvm::{
    ExactScoreboard, ReplayToken, Seed, TestDescriptor, TestOutcome, TestRunConfig, Testbench,
};

use crate::counter::Counter;
use crate::verification::{
    counter_sequence, CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult,
    RandomCounterSequence, Result,
};

/// Number of random regression cycles.
pub const RANDOM_CYCLES: u64 = 10_000;

/// Default replay stream.
pub const DEFAULT_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x72d7_5a12_993e_8411));

/// Registry test descriptors.
pub static TESTS: &[TestDescriptor] = &[
    TestDescriptor::deterministic(
        "counter-smoke",
        "Deterministic reset, count and hold test",
        run_counter_smoke,
    )
    .with_trace(),
    TestDescriptor::replayable(
        "counter-random",
        "10 000 cycles randomized counter regression",
        run_counter_random,
    )
    .with_trace(),
];

/// Counter smoke test wrapper.
fn run_counter_smoke(config: &TestRunConfig) -> TestOutcome {
    match run_deterministic_counter(config) {
        Ok(result) => TestOutcome::from_result(&result),
        Err(error) => TestOutcome::error(error),
    }
}

/// Counter replayable random test wrapper.
fn run_counter_random(config: &TestRunConfig) -> TestOutcome {
    match run_random_counter(config) {
        Ok(result) => TestOutcome::from_result(&result),
        Err(error) => TestOutcome::error(error),
    }
}

/// Runs deterministic counter test.
fn run_deterministic_counter(config: &TestRunConfig) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    let trace_dir = tempfile::tempdir()?;
    let trace_path = config.trace_path_or(trace_dir.path().to_path_buf().join("counter-smoke.vcd"));

    dut.open_trace(&trace_path)?;

    let result = Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

/// Runs replayable random counter test.
fn run_random_counter(config: &TestRunConfig) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    let sequence = RandomCounterSequence::new(
        config.replay_token_or(DEFAULT_REPLAY),
        config.cycles_or(RANDOM_CYCLES),
    );

    let trace_dir = tempfile::tempdir()?;
    let trace_path =
        config.trace_path_or(trace_dir.path().to_path_buf().join("counter-random.vcd"));

    dut.open_trace(&trace_path)?;

    let result = Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

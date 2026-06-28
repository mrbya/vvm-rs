use vvm::{
    ExactScoreboard, ReplayToken, Seed, TestDescriptor, TestOutcome, TestRunConfig, Testbench,
};

use crate::counter::Counter;
use crate::verification::{
    counter_sequence, CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult,
    RandomCounterSequence, Result,
};

/// Default registered test.
pub const DEFAULT_TEST: &str = "counter-random";

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
    ),
    TestDescriptor::replayable(
        "counter-random",
        "10 000 cycles randomized counter regression",
        run_counter_random,
    ),
];

/// Counter smoke test wrapper.
fn run_counter_smoke(_config: &TestRunConfig) -> TestOutcome {
    match run_deterministic_counter() {
        Ok(result) => TestOutcome::from_result(&result),
        Err(error) => TestOutcome::error(error),
    }
}

/// Counter replayable random test wrapper.
fn run_counter_random(config: &TestRunConfig) -> TestOutcome {
    let replay = config.replay_token().unwrap_or(DEFAULT_REPLAY);

    match run_random_counter(replay, RANDOM_CYCLES) {
        Ok(result) => TestOutcome::from_result(&result),
        Err(error) => TestOutcome::error(error),
    }
}

/// Runs deterministic counter test.
fn run_deterministic_counter() -> Result<CounterTestResult> {
    let dut = Counter::new()?;

    let result = Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

/// Runs replayable random counter test.
fn run_random_counter(replay: ReplayToken, cycles: u64) -> Result<CounterTestResult> {
    let dut = Counter::new()?;

    let sequence = RandomCounterSequence::new(replay.seed(), cycles);

    let result = Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

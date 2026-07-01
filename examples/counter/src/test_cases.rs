use cxx::FailurePolicy;
use vvm::{ExactScoreboard, ReplayToken, Seed, TestRunConfig, Testbench};

use crate::counter::Counter;
use crate::verification::{
    counter_sequence, CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult,
    FailingReferenceModel, RandomCounterSequence, Result,
};

/// Number of random regression cycles.
pub const RANDOM_CYCLES: u64 = 10_000;

/// Default replay stream.
pub const DEFAULT_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x72d7_5a12_993e_8411));

/// Deterministic reset, count and hold test
#[vvm::test(trace)]
fn counter_smoke(config: &TestRunConfig) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

/// Intentionally failing counter test.
#[vvm::test(trace)]
fn counter_fail(config: &TestRunConfig) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(FailingReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .with_failure_policy(
            FailurePolicy::collect_up_to(10).expect("hard-coded failure policy should build"),
        )
        .run::<CounterObservation>();

    Ok(result)
}

/// 10 000 cycles randomized counter regression
#[vvm::test(
    trace,
    cycles,
    replay(default = DEFAULT_REPLAY),
)]
fn counter_random(config: &TestRunConfig) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    let sequence = RandomCounterSequence::new(
        config.replay_token_or(DEFAULT_REPLAY),
        config.cycles_or(RANDOM_CYCLES),
    );

    config.configure_trace(&mut dut)?;

    let result = Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

vvm::test_registry! {
    /// Registry test descriptors.
    pub static TESTS = [
        counter_smoke,
        counter_fail,
        counter_random,
    ];
}

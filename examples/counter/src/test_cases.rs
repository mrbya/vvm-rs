use vvm::random::{ReplayToken, Seed};
use vvm::test::TestContext;
use vvm::testbench::{ExactScoreboard, FailurePolicy, Testbench};

use crate::counter::Counter;
use crate::coverage::CounterCoverage;
use crate::verification::{
    CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult,
    FailingReferenceModel, RandomCounterSequence, Result, counter_sequence,
};

/// Number of random regression cycles.
pub const RANDOM_CYCLES: u64 = 10_000;

/// Default replay stream.
pub const DEFAULT_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x72d7_5a12_993e_8411));

/// Deterministic reset, count and hold test
#[vvm::test(trace, coverage)]
fn counter_smoke(context: &mut TestContext) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    context.config().configure_trace(&mut dut)?;

    Ok(Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .with_coverage(CounterCoverage::new("dut.counter")?)
        .run_covered::<CounterObservation>(context))
}

/// Intentionally failing counter test.
#[ignore = "intentional VVM failure-reporting example"]
#[vvm::test(trace, coverage)]
fn counter_fail(context: &mut TestContext) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    context.config().configure_trace(&mut dut)?;

    Ok(Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(FailingReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .with_failure_policy(FailurePolicy::collect_up_to(10)?)
        .with_coverage(CounterCoverage::new("dut.counter")?)
        .run_covered::<CounterObservation>(context))
}

/// 10 000 cycles randomized counter regression
#[vvm::test(
    trace,
    cycles,
    replay(default = DEFAULT_REPLAY),
    coverage,
)]
fn counter_random(context: &mut TestContext) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    let sequence = RandomCounterSequence::new(
        context.config().replay_token_or(DEFAULT_REPLAY),
        context.config().cycles_or(RANDOM_CYCLES),
    );

    context.config().configure_trace(&mut dut)?;

    Ok(Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .with_coverage(CounterCoverage::new("dut.counter")?)
        .run_covered::<CounterObservation>(context))
}

#[cfg(test)]
mod tests {
    use super::__vvm_test_descriptor_counter_smoke;

    #[test]
    fn descriptor_captures_counter_coverage() -> Result<(), Box<dyn std::error::Error>> {
        let run = __vvm_test_descriptor_counter_smoke.run(&vvm::test::TestRunConfig::new())?;
        let coverage = run.coverage().ok_or("missing coverage snapshot")?;

        assert_eq!(coverage.groups().len(), 1);
        assert!(coverage.group("dut.counter").is_some());

        Ok(())
    }
}

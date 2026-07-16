use vvm::{ExactScoreboard, TestRunConfig, Testbench};

use crate::multi_clock_counter::MultiClockCounter;
use crate::verification::{
    clock_scheduler, multi_clock_sequence, MultiClockObservation, MultiClockReferenceModel,
    MultiClockTestResult, Result,
};

#[vvm::test(trace)]
/// Verifies independently timed core and peripheral clocks.
fn multi_clock_smoke(config: &TestRunConfig) -> Result<MultiClockTestResult> {
    let mut dut = MultiClockCounter::new()?;

    config.configure_trace(&mut dut)?;

    Ok(Testbench::new(dut)
        .with_sequence(multi_clock_sequence())
        .with_reference_model(MultiClockReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clocks(clock_scheduler()?)
        .run::<MultiClockObservation>())
}

#[test]
fn runner_verifies_independent_clocks() -> Result<()> {
    let result = Testbench::new(MultiClockCounter::new()?)
        .with_sequence(multi_clock_sequence())
        .with_reference_model(MultiClockReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clocks(clock_scheduler()?)
        .run::<MultiClockObservation>();

    assert!(result.passed());
    assert_eq!(result.cycles(), 8);
    assert_eq!(result.checks(), 8);
    assert_eq!(result.final_time().ticks(), 33);

    Ok(())
}

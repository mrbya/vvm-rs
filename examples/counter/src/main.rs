//! Counter verification using the synchronous VVM runner.

use vvm_core::{ExactScoreboard, Testbench};

use crate::generated::Counter;
use crate::verification::{
    CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult, Error, Result,
    counter_sequence,
};

/// Generated DUT wrapper.
mod generated {
    include!(concat!(env!("OUT_DIR"), "/vvm/counter/generated/dut.rs"));
}

/// Counter-specific verification setup.
mod verification;

fn main() -> Result<()> {
    let result = run_simulation()?;

    print_result(&result);

    if result.passed() {
        println!("counter verification completed successfully");

        return Ok(());
    }

    Err(Error::TestFailed)
}

/// Runs the counter testbench.
fn run_simulation() -> Result<CounterTestResult> {
    let dut = Counter::new()?;

    let result = Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

/// Prints compact and detailed verification results.
fn print_result(result: &CounterTestResult) {
    println!("{result}");

    for failure in result.failures() {
        eprintln!("check failure: {failure}");
    }

    if let Some(error) = result.simulation_error() {
        eprintln!("simulation error: {error}");
    }

    if let Some(error) = result.finalization_error() {
        eprintln!("finalization error: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{Result, run_simulation};

    #[test]
    fn runner_verifies_counter() -> Result<()> {
        let result = run_simulation()?;

        assert!(result.passed());
        assert_eq!(result.cycles(), 7);
        assert_eq!(result.checks(), 7);
        assert_eq!(result.failure_count(), 0);
        assert!(result.simulation_error().is_none());
        assert!(result.finalization_error().is_none());

        Ok(())
    }
}

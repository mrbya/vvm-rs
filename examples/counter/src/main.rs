//! Counter verification using the synchronous VVM runner.

use std::path::Path;

use vvm::{ExactScoreboard, Seed, Testbench};

use crate::counter::Counter;
use crate::verification::{
    CounterClock, CounterObservation, CounterReferenceModel, CounterTestResult, Error,
    RandomCounterSequence, Result,
};

vvm::include_dut!(counter);

/// Counter-specific verification setup.
mod verification;

fn main() -> Result<()> {
    let trace_dir = tempfile::tempdir()?;
    let trace_path = trace_dir.path().join("counter.vcd");

    let result = run_simulation(Some(&trace_path))?;

    print_result(&result)
}

/// Runs the counter testbench.
fn run_simulation(trace_path: Option<&Path>) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    if let Some(path) = trace_path {
        dut.open_trace(path)?;
    }

    let seed = Seed::new(0x72d7_5a12_993e_8411);
    let sequence = RandomCounterSequence::new(seed, 10_000);

    let result = Testbench::new(dut)
        .with_replayable_sequence(sequence)
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(CounterClock)
        .run::<CounterObservation>();

    Ok(result)
}

/// Prints compact and detailed verification results.
fn print_result(result: &CounterTestResult) -> Result<()> {
    if result.passed() {
        println!("{}", result.summary());
        println!("counter verification completed successfully");

        return Ok(());
    }

    eprintln!("{}", result.detailed_report());

    Err(Error::TestFailed)
}

#[cfg(test)]
mod tests {
    use super::{Result, run_simulation};

    #[test]
    fn runner_verifies_counter() -> Result<()> {
        let result = run_simulation(None)?;

        assert!(result.passed());
        assert_eq!(result.cycles(), 10000);
        assert_eq!(result.checks(), 10000);
        assert_eq!(result.final_time(), vvm::SimulationTime::from_ticks(20000));
        assert_eq!(result.failure_count(), 0);
        assert!(result.simulation_error().is_none());
        assert!(result.finalization_error().is_none());

        Ok(())
    }

    #[test]
    fn generates_complete_counter_vcd() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let trace_path = directory.path().join("counter.vcd");

        let result = run_simulation(Some(&trace_path))?;

        assert!(result.passed());

        let metadata = std::fs::metadata(&trace_path)?;

        assert!(metadata.is_file());
        assert!(metadata.len() > 0);

        let contents = std::fs::read_to_string(&trace_path)?;

        assert!(contents.contains("$enddefinitions"));
        assert!(contents.contains("$var"));
        assert!(contents.contains("#0"));

        let timestamps = contents
            .lines()
            .filter_map(|line| line.strip_prefix('#'))
            .map(str::parse::<u64>)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        assert_eq!(timestamps.first().copied(), Some(0));

        assert_eq!(timestamps.last().copied(), Some(20000));

        assert!(
            timestamps
                .windows(2)
                .all(|pair| { pair.first() < pair.get(1) })
        );

        Ok(())
    }
}

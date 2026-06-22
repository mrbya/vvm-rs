//! Manual simulation using the generated Verilated counter wrapper.

use vvm_core::{Drive, Dut, ExactScoreboard, ReferenceModel, Sample, Scoreboard};

use crate::generated::Counter;
use crate::verification::{CounterObservation, CounterReferenceModel, Result, counter_sequence};

/// Generated DUT wrapper.
mod generated {
    include!(concat!(env!("OUT_DIR"), "/vvm/counter/generated/dut.rs"));
}
/// Counter-specific verification setup.
mod verification;

fn main() -> Result<()> {
    run_simulation()
}

/// Runs the explicit counter verification loop.
fn run_simulation() -> Result<()> {
    let mut dut = Counter::new()?;
    let mut reference_model = CounterReferenceModel::default();
    let mut scoreboard = ExactScoreboard;

    for (cycle, stimulus) in counter_sequence().enumerate() {
        // 1. Drive clock low.
        dut.set_clk(false)?;

        // 2. Drive stimulus.
        stimulus.drive(&mut dut)?;

        // 3. Evaluate low phase.
        Dut::evaluate(&mut dut)?;

        // 4. Drive the active edge.
        dut.set_clk(true)?;

        // 5. Evaluate active edge.
        Dut::evaluate(&mut dut)?;

        // 6. Sample post-edge outputs.
        let observed = CounterObservation::sample(&dut)?;

        // 7. Update the model for the active edge.
        let expected = reference_model.predict(&stimulus);

        // 8. Compare expected and observed state.
        scoreboard.check(expected, observed)?;

        println!(
            "cycle {cycle:02}: stimulus={stimulus:?} expected={}, observed={}",
            expected.count(),
            observed.count(),
        );
    }

    Dut::finalize(&mut dut)?;

    println!("manual counter verification completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generated::{Counter, CounterError, Result};
    use super::run_simulation;

    #[test]
    fn methodology_loop_verifies_counter() -> crate::verification::Result<()> {
        run_simulation()
    }

    #[test]
    fn constructs_generated_dut() -> Result<()> {
        let counter = Counter::new()?;

        assert!(!counter.is_finished());

        Ok(())
    }

    #[test]
    fn finish_is_idempotent() -> Result<()> {
        let mut counter = Counter::new()?;

        counter.finish()?;
        counter.finish()?;

        assert!(counter.is_finished());

        Ok(())
    }

    #[test]
    fn rejects_evaluation_after_finish() -> Result<()> {
        let mut counter = Counter::new()?;

        counter.finish()?;

        assert_eq!(counter.eval(), Err(CounterError::Finished));

        Ok(())
    }

    #[test]
    fn rejects_input_drive_after_finish() -> Result<()> {
        let mut counter = Counter::new()?;

        counter.finish()?;

        assert_eq!(counter.set_enable(true), Err(CounterError::Finished));

        Ok(())
    }

    #[test]
    fn rejects_output_sampling_after_finish() -> Result<()> {
        let mut counter = Counter::new()?;

        counter.finish()?;

        assert_eq!(counter.count(), Err(CounterError::Finished));

        Ok(())
    }

    #[test]
    fn dropping_unfinished_dut_is_safe() -> Result<()> {
        let counter = Counter::new()?;

        drop(counter);

        Ok(())
    }

    #[test]
    fn dropping_explicitly_finished_dut_is_safe() -> Result<()> {
        let mut counter = Counter::new()?;

        counter.finish()?;
        drop(counter);

        Ok(())
    }

    #[test]
    fn debug_output_hides_native_details() -> Result<()> {
        let counter = Counter::new()?;
        let debug = format!("{counter:?}");

        assert!(debug.contains("Counter"));
        assert!(debug.contains("finished"));
        assert!(!debug.contains("UniquePtr"));
        assert!(!debug.contains("Vcounter"));

        Ok(())
    }
}

//! Caller-owned tri-state bus resolution example.

// Include the generated wrapper rather than exposing CXX or Verilator details.
#[cfg(test)]
vvm::include_dut!(tri_state_bus);

/// Caller-owned inout resolution and settling policy.
#[cfg(test)]
mod resolution;

/// Typed generated-DUT drive and observation definitions.
#[cfg(test)]
mod verification;

/// Vertical inout-resolution integration scenarios.
#[cfg(test)]
mod tests {
    use std::error::Error;

    use vvm::dut::{Drive, Dut, Sample};
    use vvm::timing::{SimulationTime, TimeStep};

    use crate::resolution::{BusDriver, SettleError, settle_bus};
    use crate::tri_state_bus::TriStateBus;
    use crate::verification::{BusObservation, DutDriverControl};

    /// Settles one configured drive state and samples its result.
    fn settle_and_sample(
        dut: &mut TriStateBus,
        controls: DutDriverControl,
        external: BusDriver,
        floating_value: u8,
    ) -> Result<(crate::resolution::SettleRun, BusObservation), Box<dyn Error>> {
        // First apply DUT-owned controls; external resolution happens in Rust.
        controls.drive(dut)?;

        // Settling repeatedly evaluates the feedback path at one logical time.
        let run = settle_bus(dut, external, floating_value, 8)?;
        let observation = BusObservation::sample(dut)?;

        Ok((run, observation))
    }

    #[test]
    fn resolves_floating_pull_up_vertically() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let run = settle_bus(&mut dut, BusDriver::released(), 0xFF, 8)?;
        let observation = BusObservation::sample(&dut)?;

        assert_eq!(run.resolution().value(), 0xFF);
        assert_eq!(run.resolution().driven_mask(), 0);
        assert_eq!(run.resolution().floating_mask(), 0xFF);
        assert_eq!(run.evaluations(), 2);
        assert_eq!(*observation.data().input(), 0xFF);
        assert_eq!(*observation.data().output_enable(), 0);
        assert_eq!(observation.sampled_data(), 0xFF);

        Ok(())
    }

    #[test]
    fn resolves_external_driver_vertically() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let (run, observation) = settle_and_sample(
            &mut dut,
            DutDriverControl::released(),
            BusDriver::new(0xFF, 0xA5),
            0,
        )?;

        assert_eq!(run.resolution().value(), 0xA5);
        assert_eq!(run.resolution().driven_mask(), 0xFF);
        assert_eq!(run.resolution().floating_mask(), 0);
        assert_eq!(*observation.data().input(), 0xA5);
        assert_eq!(*observation.data().output_enable(), 0);
        assert_eq!(observation.sampled_data(), 0xA5);

        Ok(())
    }

    #[test]
    fn resolves_dut_driver_vertically() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let (run, observation) = settle_and_sample(
            &mut dut,
            DutDriverControl::new(0xFF, 0x3C),
            BusDriver::released(),
            0,
        )?;

        assert_eq!(run.resolution().value(), 0x3C);
        assert_eq!(run.resolution().driven_mask(), 0xFF);
        assert_eq!(run.resolution().floating_mask(), 0);
        assert_eq!(*observation.data().input(), 0x3C);
        assert_eq!(*observation.data().output_enable(), 0xFF);
        assert_eq!(*observation.data().output_value(), 0x3C);
        assert_eq!(observation.sampled_data(), 0x3C);

        Ok(())
    }

    #[test]
    fn resolves_split_ownership_vertically() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let (run, observation) = settle_and_sample(
            &mut dut,
            DutDriverControl::new(0xF0, 0xA0),
            BusDriver::new(0x0F, 0x05),
            0,
        )?;

        assert_eq!(run.resolution().value(), 0xA5);
        assert_eq!(run.resolution().driven_mask(), 0xFF);
        assert_eq!(run.resolution().floating_mask(), 0);
        assert_eq!(*observation.data().input(), 0xA5);
        assert_eq!(*observation.data().output_enable(), 0xF0);
        assert_eq!(*observation.data().output_value(), 0xA0);
        assert_eq!(observation.sampled_data(), 0xA5);

        Ok(())
    }

    #[test]
    fn accepts_matching_overlap_vertically() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let (run, observation) = settle_and_sample(
            &mut dut,
            DutDriverControl::new(0x0F, 0x05),
            BusDriver::new(0x03, 0x01),
            0,
        )?;

        assert_eq!(run.resolution().value(), 0x05);
        assert_eq!(run.resolution().driven_mask(), 0x0F);
        assert_eq!(run.resolution().floating_mask(), 0xF0);
        assert_eq!(*observation.data().input(), 0x05);
        assert_eq!(observation.sampled_data(), 0x05);

        Ok(())
    }

    #[test]
    fn reports_contention_before_input_mutation() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let stable = settle_bus(&mut dut, BusDriver::released(), 0xFF, 8)?;
        let previous_input = dut.data_input()?;
        let previous_time = dut.simulation_time();

        DutDriverControl::new(0x0F, 0x0A).drive(&mut dut)?;

        let result = settle_bus(&mut dut, BusDriver::new(0x0F, 0x05), 0xFF, 8);

        assert_eq!(stable.resolution().value(), previous_input);
        assert!(matches!(result, Err(SettleError::Contention(error)) if error.mask() == 0x0F));
        assert_eq!(dut.data_input()?, previous_input);
        assert_eq!(dut.simulation_time(), previous_time);
        assert!(!dut.is_finished());

        Ok(())
    }

    #[test]
    fn reports_evaluation_limit_after_writing_changed_input() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;

        let result = settle_bus(&mut dut, BusDriver::released(), 0xFF, 1);

        assert!(matches!(
            result,
            Err(SettleError::EvaluationLimitExceeded { limit: 1 })
        ));
        assert_eq!(dut.data_input()?, 0xFF);

        Ok(())
    }

    #[test]
    fn rejects_zero_evaluation_limit_without_mutation() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        let input = dut.data_input()?;
        let time = dut.simulation_time();

        let result = settle_bus(&mut dut, BusDriver::released(), 0xFF, 0);

        assert!(matches!(result, Err(SettleError::InvalidEvaluationLimit)));
        assert_eq!(dut.data_input()?, input);
        assert_eq!(dut.simulation_time(), time);

        Ok(())
    }

    #[test]
    fn propagates_finished_dut_error() -> Result<(), Box<dyn Error>> {
        let mut dut = TriStateBus::new()?;
        dut.finish()?;

        let result = settle_bus(&mut dut, BusDriver::released(), 0xFF, 8);

        assert!(matches!(result, Err(SettleError::Dut(_))));

        Ok(())
    }

    #[test]
    fn traces_resolved_bus_phases() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let trace_path = directory.path().join("tri_state_bus.vcd");
        let mut dut = TriStateBus::new()?;

        dut.open_trace(&trace_path)?;

        settle_and_sample(
            &mut dut,
            DutDriverControl::released(),
            BusDriver::released(),
            0xFF,
        )?;

        Dut::advance_time(&mut dut, TimeStep::ONE)?;

        settle_and_sample(
            &mut dut,
            DutDriverControl::released(),
            BusDriver::new(0xFF, 0xA5),
            0,
        )?;

        Dut::advance_time(&mut dut, TimeStep::ONE)?;

        settle_and_sample(
            &mut dut,
            DutDriverControl::new(0xFF, 0x3C),
            BusDriver::released(),
            0,
        )?;

        Dut::advance_time(&mut dut, TimeStep::ONE)?;

        settle_and_sample(
            &mut dut,
            DutDriverControl::new(0xF0, 0xA0),
            BusDriver::new(0x0F, 0x05),
            0,
        )?;

        assert_eq!(dut.simulation_time(), SimulationTime::from_ticks(3));
        assert!(dut.trace_is_open());

        dut.finish()?;

        assert!(!dut.trace_is_open());

        let metadata = std::fs::metadata(&trace_path)?;
        assert!(metadata.is_file());
        assert!(metadata.len() > 0);

        let contents = std::fs::read_to_string(&trace_path)?;
        assert!(contents.contains("$enddefinitions"));
        assert!(contents.contains("$var"));
        assert!(contents.contains("drive_enable"));
        assert!(contents.contains("drive_value"));
        assert!(contents.contains("data"));

        let timestamps = contents
            .lines()
            .filter_map(|line| line.strip_prefix('#'))
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()?;

        let unique_timestamps = timestamps
            .into_iter()
            .fold(Vec::new(), |mut unique, timestamp| {
                if unique.last().copied() != Some(timestamp) {
                    unique.push(timestamp);
                }

                unique
            });

        assert_eq!(unique_timestamps, vec![0, 1, 2, 3]);
        assert!(unique_timestamps.windows(2).all(|pair| match *pair {
            [previous, next] => previous < next,
            _ => false,
        }));

        Ok(())
    }
}

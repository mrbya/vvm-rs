//! Minimal bidirectional-line fixture for documentation.

#[cfg(test)]
vvm::include_dut!(line_adapter);

#[cfg(test)]
mod tests {
    use vvm::dut::{InoutState, Sample as SampleFromDut};

    use crate::line_adapter::LineAdapter;

    // ANCHOR: observation
    #[derive(Debug, Clone, Copy, PartialEq, Eq, vvm::Sample)]
    #[vvm(dut = crate::line_adapter::LineAdapter)]
    struct LineObservation {
        #[vvm(port)]
        line: InoutState<bool>,

        #[vvm(port)]
        sampled_line: bool,
    }
    // ANCHOR_END: observation

    // ANCHOR: resolution
    fn resolve_line(state: &InoutState<bool>, external_drive: Option<bool>, floating: bool) -> bool {
        match (external_drive, *state.output_enable()) {
            (Some(external), true) if external != *state.output_value() => {
                panic!("line contention between DUT and external driver")
            }
            (Some(external), _) => external,
            (None, true) => *state.output_value(),
            (None, false) => floating,
        }
    }
    // ANCHOR_END: resolution

    #[test]
    fn caller_resolves_bidirectional_line() -> Result<(), Box<dyn std::error::Error>> {
        let mut dut = LineAdapter::new()?;

        dut.set_drive_enable(true)?;
        dut.set_drive_value(true)?;
        dut.set_line_input(false)?;
        dut.eval()?;

        let state = dut.line()?;
        let resolved = resolve_line(&state, None, false);

        assert!(resolved);

        dut.set_line_input(resolved)?;
        dut.eval()?;

        let observation = LineObservation::sample(&dut)?;

        assert!(observation.sampled_line);

        Ok(())
    }
}

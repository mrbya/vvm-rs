use crate::{Drive, Dut, ExactScoreboard, ReferenceModel, Sample, Scoreboard};

/// Error returned by the mock DUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockError {
    /// The mock has already been finalized.
    Finished,
}

/// Minimal pure-Rust DUT.
#[derive(Debug, Default)]
struct MockDut {
    /// Driven input.
    input: u8,

    /// Evaluated output.
    output: u8,

    /// Lifecycle state.
    finished: bool,
}

impl Dut for MockDut {
    type Error = MockError;

    fn eval(&mut self) -> Result<(), Self::Error> {
        if self.finished {
            return Err(MockError::Finished);
        }

        self.output = self.input;

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        self.finished = true;

        Ok(())
    }
}

/// Input value for the mock DUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockStimulus {
    /// Value driven into the mock.
    value: u8,
}

impl Drive<MockDut> for MockStimulus {
    fn drive(&self, dut: &mut MockDut) -> Result<(), <MockDut as Dut>::Error> {
        if dut.finished {
            return Err(MockError::Finished);
        }

        dut.input = self.value;

        Ok(())
    }
}

/// Sampled mock output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockObservation {
    /// Sampled value.
    value: u8,
}

impl Sample<MockDut> for MockObservation {
    fn sample(dut: &MockDut) -> Result<Self, <MockDut as Dut>::Error> {
        if dut.finished {
            return Err(MockError::Finished);
        }

        Ok(Self { value: dut.output })
    }
}

/// Stateless mock reference model.
struct MockReferenceModel;

impl ReferenceModel<MockStimulus> for MockReferenceModel {
    type Expected = MockObservation;

    fn predict(&mut self, stimulus: &MockStimulus) -> Self::Expected {
        MockObservation {
            value: stimulus.value,
        }
    }
}

#[test]
fn drives_samples_and_checks_mock_dut() -> Result<(), MockError> {
    let stimulus = MockStimulus { value: 42 };
    let mut dut = MockDut::default();
    let mut model = MockReferenceModel;
    let mut scoreboard = ExactScoreboard;

    stimulus.drive(&mut dut)?;
    Dut::eval(&mut dut)?;

    let observed = MockObservation::sample(&dut)?;
    let expected = model.predict(&stimulus);

    scoreboard
        .check(expected, observed)
        .expect("SB check should pass");

    Dut::finish(&mut dut)?;

    Dut::eval(&mut dut).expect_err("Dut should be finished");

    Ok(())
}

#[test]
fn exact_scoreboard_retains_mismatched_values() {
    let mut scoreboard = ExactScoreboard;

    let result = scoreboard.check(4, 7).expect_err("should throw mismatch");

    assert_eq!(result.expected(), &4);
    assert_eq!(result.observed(), &7);
}

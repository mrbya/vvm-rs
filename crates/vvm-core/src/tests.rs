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

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        if self.finished {
            return Err(MockError::Finished);
        }

        self.output = self.input;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        self.finished = true;

        Ok(())
    }
}

/// Clock driver for the combinational mock DUT.
#[derive(Debug, Default)]
struct MockClock;

impl crate::Clock<MockDut> for MockClock {
    fn drive_inactive(&mut self, _dut: &mut MockDut) -> Result<(), MockError> {
        Ok(())
    }

    fn drive_active(&mut self, _dut: &mut MockDut) -> Result<(), MockError> {
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
    Dut::evaluate(&mut dut)?;

    let observed = MockObservation::sample(&dut)?;
    let expected = model.predict(&stimulus);

    scoreboard
        .check(expected, observed)
        .expect("SB check should pass");

    Dut::finalize(&mut dut)?;

    Dut::evaluate(&mut dut).expect_err("Dut should be finished");

    Ok(())
}

#[test]
fn exact_scoreboard_retains_mismatched_values() {
    let mut scoreboard = ExactScoreboard;

    let result = scoreboard.check(4, 7).expect_err("should throw mismatch");

    assert_eq!(result.expected(), &4);
    assert_eq!(result.observed(), &7);
}

#[test]
fn runner_executes_pure_rust_mock() {
    let sequence = [
        MockStimulus { value: 4 },
        MockStimulus { value: 8 },
        MockStimulus { value: 15 },
        MockStimulus { value: 16 },
        MockStimulus { value: 23 },
        MockStimulus { value: 42 },
    ];

    let result = crate::Testbench::new(MockDut::default())
        .with_sequence(sequence)
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run::<MockObservation>();

    assert!(result.passed());
    assert_eq!(result.cycles(), 6);
    assert_eq!(result.checks(), 6);
    assert_eq!(result.failure_count(), 0);
    assert!(result.simulation_error().is_none());
    assert!(result.finalization_error().is_none());
}

/// Reference model that intentionally predicts the wrong value.
struct IncorrectReferenceModel;

impl ReferenceModel<MockStimulus> for IncorrectReferenceModel {
    type Expected = MockObservation;

    fn predict(&mut self, stimulus: &MockStimulus) -> Self::Expected {
        MockObservation {
            value: stimulus.value.wrapping_add(1),
        }
    }
}

#[test]
fn runner_collects_only_configured_failure_count() -> Result<(), crate::InvalidFailureLimit> {
    let result = crate::Testbench::new(MockDut::default())
        .with_sequence([
            MockStimulus { value: 1 },
            MockStimulus { value: 2 },
            MockStimulus { value: 3 },
            MockStimulus { value: 4 },
        ])
        .with_reference_model(IncorrectReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .with_failure_policy(crate::FailurePolicy::collect_up_to(2)?)
        .run::<MockObservation>();

    assert!(!result.passed());
    assert_eq!(result.cycles(), 2);
    assert_eq!(result.checks(), 2);
    assert_eq!(result.failure_count(), 2);
    assert!(result.stopped_by_failure_policy());

    let failures = result.failures();

    assert_eq!(failures.first().map(crate::CheckFailure::cycle), Some(0));

    assert_eq!(failures.get(1).map(crate::CheckFailure::cycle), Some(1));

    Ok(())
}

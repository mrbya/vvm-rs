use std::cell::Cell;
use std::rc::Rc;

use crate::{
    CheckFailure, Clock, ClockScheduler, ClockTiming, CycleTiming, Drive, Dut, ExactScoreboard,
    InvalidTimeStep, ReferenceModel, Sample, Scoreboard, SimulationStage, SimulationTime, TimeStep,
    TimedDut,
};

/// Error returned by the mock DUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockError {
    /// The mock has already been finalized.
    Finished,

    /// Advancing time would overflow the simulation clock.
    TimeOverflow,
}

/// Minimal pure-Rust DUT.
#[derive(Debug, Default)]
struct MockDut {
    /// Driven input.
    input: u8,

    /// Evaluated output.
    output: u8,

    /// Current simulation time.
    time: SimulationTime,

    /// Lifecycle state.
    finished: bool,
}

impl MockDut {
    /// Constructs a mock DUT at an explicit simulation time.
    const fn at_time(time: SimulationTime) -> Self {
        Self {
            input: 0,
            output: 0,
            time,
            finished: false,
        }
    }
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

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        if self.finished {
            return Err(MockError::Finished);
        }

        let Some(next_time) = self.time.checked_add(delta) else {
            return Err(MockError::TimeOverflow);
        };

        self.time = next_time;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        self.finished = true;

        Ok(())
    }
}

/// Mock DUT with an explicitly queryable delayed-event queue.
struct MockTimedDut {
    /// Current time.
    time: SimulationTime,
    /// Whether an event is pending.
    pending: bool,
    /// Absolute next event time.
    next: Option<SimulationTime>,
}

impl Dut for MockTimedDut {
    type Error = MockError;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        let Some(next) = self.time.checked_add(delta) else {
            return Err(MockError::TimeOverflow);
        };

        self.time = next;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl TimedDut for MockTimedDut {
    fn events_pending(&self) -> Result<bool, Self::Error> {
        Ok(self.pending)
    }

    fn next_time_slot(&self) -> Result<Option<SimulationTime>, Self::Error> {
        Ok(self.next)
    }
}

fn assert_timed_dut<T: TimedDut>() {}

#[test]
fn timed_dut_queries_return_absolute_time() -> Result<(), MockError> {
    assert_timed_dut::<MockTimedDut>();

    let dut = MockTimedDut {
        time: SimulationTime::ZERO,
        pending: true,
        next: Some(SimulationTime::from_ticks(2)),
    };

    assert!(dut.events_pending()?);
    assert_eq!(dut.next_time_slot()?, Some(SimulationTime::from_ticks(2)));

    Ok(())
}

#[test]
fn timed_dut_can_report_no_event() -> Result<(), MockError> {
    let dut = MockTimedDut {
        time: SimulationTime::ZERO,
        pending: false,
        next: None,
    };

    assert!(!dut.events_pending()?);
    assert_eq!(dut.next_time_slot()?, None);

    Ok(())
}

/// Minimal overflow-test DUT that exposes finalization through a shared flag.
#[derive(Debug)]
struct OverflowMockDut {
    /// Current simulation time.
    time: SimulationTime,

    /// Lifecycle state.
    finished: bool,

    /// Records whether finalization ran.
    finalized: Rc<Cell<bool>>,
}

impl OverflowMockDut {
    /// Constructs an overflow-test DUT at a chosen time.
    fn at_time(time: SimulationTime, finalized: Rc<Cell<bool>>) -> Self {
        Self {
            time,
            finished: false,
            finalized,
        }
    }
}

impl Dut for OverflowMockDut {
    type Error = MockError;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        if self.finished {
            return Err(MockError::Finished);
        }

        Ok(())
    }

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        if self.finished {
            return Err(MockError::Finished);
        }

        let Some(next_time) = self.time.checked_add(delta) else {
            return Err(MockError::TimeOverflow);
        };

        self.time = next_time;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        self.finished = true;
        self.finalized.set(true);

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

impl Clock<OverflowMockDut> for MockClock {
    fn drive_inactive(&mut self, _dut: &mut OverflowMockDut) -> Result<(), MockError> {
        Ok(())
    }

    fn drive_active(&mut self, _dut: &mut OverflowMockDut) -> Result<(), MockError> {
        Ok(())
    }
}

/// Secondary clock driver for scheduler integration tests.
struct SecondaryClock;

impl Clock<MockDut> for SecondaryClock {
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

impl Drive<OverflowMockDut> for MockStimulus {
    fn drive(&self, dut: &mut OverflowMockDut) -> Result<(), <OverflowMockDut as Dut>::Error> {
        if dut.finished {
            return Err(MockError::Finished);
        }

        Ok(())
    }
}

impl Sample<OverflowMockDut> for MockObservation {
    fn sample(_dut: &OverflowMockDut) -> Result<Self, <OverflowMockDut as Dut>::Error> {
        Ok(Self { value: 0 })
    }
}

impl ReferenceModel<MockStimulus> for Rc<Cell<u8>> {
    type Expected = MockObservation;

    fn predict(&mut self, _stimulus: &MockStimulus) -> Self::Expected {
        MockObservation { value: self.get() }
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
fn dut_lifecycle_separates_evaluation_and_time_advancement() -> Result<(), MockError> {
    let mut dut = MockDut::default();
    let stimulus = MockStimulus { value: 7 };
    let initial_time = Dut::simulation_time(&dut);

    stimulus.drive(&mut dut)?;
    Dut::evaluate(&mut dut)?;

    assert_eq!(Dut::simulation_time(&dut), initial_time);
    assert_eq!(dut.output, 7);

    Dut::advance_time(&mut dut, TimeStep::ONE)?;

    assert_eq!(Dut::simulation_time(&dut), SimulationTime::from_ticks(1));

    Dut::finalize(&mut dut)?;

    assert_eq!(Dut::simulation_time(&dut), SimulationTime::from_ticks(1));
    assert_eq!(
        Dut::advance_time(&mut dut, TimeStep::ONE),
        Err(MockError::Finished)
    );

    Ok(())
}

#[test]
fn finalized_dut_retains_time_but_rejects_advancement() -> Result<(), MockError> {
    let mut dut = MockDut::at_time(SimulationTime::ZERO);

    Dut::advance_time(&mut dut, TimeStep::ONE)?;
    Dut::finalize(&mut dut)?;

    assert_eq!(Dut::simulation_time(&dut), SimulationTime::from_ticks(1));
    assert_eq!(
        Dut::advance_time(&mut dut, TimeStep::ONE),
        Err(MockError::Finished)
    );
    assert_eq!(Dut::simulation_time(&dut), SimulationTime::from_ticks(1));

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
    assert_eq!(result.final_time(), SimulationTime::from_ticks(12));
    assert_eq!(result.failure_count(), 0);
    assert!(result.simulation_error().is_none());
    assert!(result.finalization_error().is_none());
}

#[test]
fn testbench_observer_receives_successful_transactions_in_order() {
    let mut observed_cycles = Vec::new();
    let result = crate::Testbench::new(MockDut::default())
        .with_sequence([
            MockStimulus { value: 4 },
            MockStimulus { value: 8 },
            MockStimulus { value: 15 },
        ])
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run_with_observer::<MockObservation, _>(|cycle| {
            observed_cycles.push((
                cycle.cycle(),
                cycle.time(),
                cycle.stimulus().value,
                cycle.observed().value,
            ));
        });

    assert!(result.passed());
    assert_eq!(
        observed_cycles,
        [
            (0, SimulationTime::from_ticks(1), 4, 4),
            (1, SimulationTime::from_ticks(3), 8, 8),
            (2, SimulationTime::from_ticks(5), 15, 15),
        ]
    );
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
    assert_eq!(result.final_time(), SimulationTime::from_ticks(3));

    let failures = result.failures();

    assert_eq!(failures.first().map(crate::CheckFailure::cycle), Some(0));
    assert_eq!(
        failures.first().map(CheckFailure::time),
        Some(SimulationTime::from_ticks(1))
    );

    assert_eq!(failures.get(1).map(crate::CheckFailure::cycle), Some(1));
    assert_eq!(
        failures.get(1).map(CheckFailure::time),
        Some(SimulationTime::from_ticks(3))
    );

    Ok(())
}

#[test]
fn testbench_observer_runs_before_failure_policy_stops() {
    let mut cycles = Vec::new();
    let result = crate::Testbench::new(MockDut::default())
        .with_sequence([
            MockStimulus { value: 1 },
            MockStimulus { value: 2 },
            MockStimulus { value: 3 },
        ])
        .with_reference_model(IncorrectReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run_with_observer::<MockObservation, _>(|cycle| cycles.push(cycle.cycle()));

    assert!(!result.passed());
    assert_eq!(result.failure_count(), 1);
    assert_eq!(cycles, [0]);
}

#[test]
fn testbench_run_delegates_to_observer_execution() {
    let sequence = [MockStimulus { value: 4 }, MockStimulus { value: 8 }];
    let ordinary = crate::Testbench::new(MockDut::default())
        .with_sequence(sequence)
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run::<MockObservation>();
    let observed = crate::Testbench::new(MockDut::default())
        .with_sequence(sequence)
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run_with_observer::<MockObservation, _>(|_| {});

    assert_eq!(ordinary.cycles(), observed.cycles());
    assert_eq!(ordinary.checks(), observed.checks());
    assert_eq!(ordinary.final_time(), observed.final_time());
    assert_eq!(ordinary.failure_count(), observed.failure_count());
    assert_eq!(ordinary.replay_token(), observed.replay_token());
}

#[test]
fn runner_uses_configured_cycle_timing() -> Result<(), InvalidTimeStep> {
    let timing = CycleTiming::new(TimeStep::new(2)?, TimeStep::new(3)?);

    let result = crate::Testbench::new(MockDut::default())
        .with_sequence([MockStimulus { value: 1 }, MockStimulus { value: 2 }])
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .with_cycle_timing(timing)
        .run::<MockObservation>();

    assert!(result.passed());
    assert_eq!(result.final_time(), SimulationTime::from_ticks(10));

    Ok(())
}

#[test]
fn multi_clock_runner_counts_primary_transactions_only() -> Result<(), Box<dyn std::error::Error>> {
    let primary = ClockTiming::new(
        CycleTiming::new(TimeStep::new(3)?, TimeStep::new(1)?),
        TimeStep::new(4)?,
    );
    let secondary = ClockTiming::from_cycle(CycleTiming::UNIT);
    let clocks = ClockScheduler::new("core", MockClock, primary)?.with_clock(
        "peripheral",
        SecondaryClock,
        secondary,
    )?;
    let result = crate::Testbench::new(MockDut::default())
        .with_sequence([MockStimulus { value: 1 }])
        .with_reference_model(MockReferenceModel)
        .with_scoreboard(ExactScoreboard)
        .with_clocks(clocks)
        .run::<MockObservation>();

    assert!(result.passed());
    assert_eq!(result.cycles(), 1);
    assert_eq!(result.checks(), 1);
    assert_eq!(result.final_time(), SimulationTime::from_ticks(5));

    Ok(())
}

#[test]
fn runner_reports_time_overflow_and_still_finalizes() {
    let finalized = Rc::new(Cell::new(false));
    let dut = OverflowMockDut::at_time(SimulationTime::from_ticks(u64::MAX), Rc::clone(&finalized));
    let result = crate::Testbench::new(dut)
        .with_sequence([MockStimulus { value: 1 }])
        .with_reference_model(Rc::new(Cell::new(0)))
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run::<MockObservation>();

    let error = result.simulation_error().expect("simulation should fail");

    assert_eq!(result.cycles(), 0);
    assert_eq!(result.checks(), 0);
    assert_eq!(error.stage(), SimulationStage::AdvanceInactivePhase);
    assert_eq!(error.time(), SimulationTime::from_ticks(u64::MAX));
    assert_eq!(result.final_time(), SimulationTime::from_ticks(u64::MAX));
    assert!(finalized.get());
}

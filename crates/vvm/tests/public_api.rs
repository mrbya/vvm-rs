//! Public facade integration coverage.

use std::borrow::Borrow;

use vvm::dut::{TimedDut, TraceableDut};
use vvm::prelude::*;
use vvm::timing::{
    SchedulerError, SimulationTime, TimeStep, TimingEvent, TimingRun, TimingScheduler, TimingStage,
};

const MANUAL_REPLAY: vvm::random::ReplayToken =
    vvm::random::ReplayToken::new(vvm::random::Seed::new(0x55));

struct ManualCoverage {
    instance: vvm::coverage::CoverageGroupInstance,
    value: vvm::coverage::Coverpoint<u8>,
}

impl vvm::coverage::CoverageGroup for ManualCoverage {
    fn instance(&self) -> &vvm::coverage::CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn vvm::coverage::CoverageGroupVisitor) {
        visitor.visit(vvm::coverage::CoverageItemRef::coverpoint(&self.value));
    }
}

/// Returns a completed result from a manually driven test with coverage.
#[vvm::test(replay(default = MANUAL_REPLAY), coverage)]
fn manual_result_with_coverage(
    context: &mut vvm::test::TestContext,
) -> Result<TestResult<(), String, String>, String> {
    let mut value = vvm::coverage::Coverpoint::builder("value")
        .bin(vvm::coverage::Bin::value("one", 1_u8))
        .build()
        .map_err(|error| error.to_string())?;
    value.sample(&1).map_err(|error| error.to_string())?;

    let coverage = ManualCoverage {
        instance: vvm::coverage::CoverageGroupInstance::new("manual", "dut.manual")
            .map_err(|error| error.to_string())?,
        value,
    };
    context
        .capture_coverage(&coverage)
        .map_err(|error| error.to_string())?;

    Ok(TestResult::completed(
        SimulationTime::ZERO,
        SimulationTime::from_ticks(5),
        context.config().replay_token(),
    ))
}

#[test]
fn facade_manual_result_supports_replay_and_coverage() -> Result<(), Box<dyn std::error::Error>> {
    let run =
        __vvm_test_descriptor_manual_result_with_coverage.run(&vvm::test::TestRunConfig::new())?;
    let coverage = run.coverage().ok_or("coverage was not captured")?;

    assert!(run.passed());
    assert_eq!(run.outcome().replay_token(), Some(MANUAL_REPLAY));
    assert_eq!(
        run.outcome()
            .statistics()
            .map(vvm::test::TestStatistics::cycles),
        Some(0)
    );
    assert_eq!(coverage.groups().len(), 1);
    assert_eq!(
        coverage
            .groups()
            .first()
            .map(vvm::coverage::snapshot::CoverageGroupSnapshot::instance_path),
        Some("dut.manual")
    );

    Ok(())
}

#[test]
fn facade_exports_canonical_error_names() {
    use vvm::coverage::artifact::{IoOperation, PersistenceError};
    use vvm::coverage::merge::{MergeCountKind, MergeCounterKind, MergeError};
    use vvm::coverage::session::{SessionCountKind, SessionError};
    use vvm::coverage::{BuildError, DefinitionError, GroupError, RuntimeError, SampleError};
    use vvm::packed::{LayoutError, UnpackedIndexError, WordCountError};
    use vvm::random::{ReplayTokenParseError, SeedParseError};
    use vvm::test::RegistryError;
    use vvm::testbench::{FailureLimitError, SimulationError};
    use vvm::timing::{ClockConfigurationError, SchedulerError, TimeStepError};

    fn accepts<T>() {}

    accepts::<BuildError>();
    accepts::<DefinitionError>();
    accepts::<GroupError>();
    accepts::<RuntimeError>();
    accepts::<SampleError>();
    accepts::<IoOperation>();
    accepts::<PersistenceError>();
    accepts::<MergeCountKind>();
    accepts::<MergeCounterKind>();
    accepts::<MergeError>();
    accepts::<SessionCountKind>();
    accepts::<SessionError>();
    accepts::<LayoutError>();
    accepts::<UnpackedIndexError>();
    accepts::<WordCountError>();
    accepts::<ReplayTokenParseError>();
    accepts::<SeedParseError>();
    accepts::<RegistryError>();
    accepts::<FailureLimitError>();
    accepts::<SimulationError<MockError>>();
    accepts::<ClockConfigurationError>();
    accepts::<SchedulerError<MockError>>();
    accepts::<TimeStepError>();
}

const fn uses_types(
    _: Option<TimingEvent>,
    _: Option<TimingRun>,
    _: Option<SchedulerError<MockError>>,
    _: Option<TimingStage>,
) {
}

#[test]
fn facade_exports_timing_scheduler_types() {
    let _scheduler = TimingScheduler::new();

    uses_types(None, None, None, None);
}

#[test]
fn prelude_exports_timed_dut_and_scheduler() {
    fn use_scheduler<D: TimedDut>(dut: &D) {
        let _scheduler = TimingScheduler::new();

        let _time = dut.simulation_time();
    }

    let dut = MockDut::default();

    use_scheduler(&dut);
}

#[derive(Debug, Default)]
struct ClockState {
    current: bool,
    previous: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    #[default]
    Running,
    Finalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockError {
    Finalized,
    TimeOverflow,
}

#[derive(Debug, Default)]
struct MockDut {
    clock: ClockState,
    reset_n: bool,
    enable: bool,
    count: u8,
    time: SimulationTime,
    lifecycle: Lifecycle,
}

impl MockDut {
    fn set_clk(&mut self, value: bool) -> Result<(), MockError> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        self.clock.current = value;
        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn set_reset_n(&mut self, value: impl Borrow<bool>) -> Result<(), MockError> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        self.reset_n = *value.borrow();
        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn set_enable(&mut self, value: impl Borrow<bool>) -> Result<(), MockError> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        self.enable = *value.borrow();
        Ok(())
    }

    fn count(&self) -> Result<u8, MockError> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        Ok(self.count)
    }
}

impl Dut for MockDut {
    type Error = MockError;

    fn evaluate(&mut self) -> Result<(), Self::Error> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        if !self.reset_n {
            self.count = 0;
        } else if !self.clock.previous && self.clock.current && self.enable {
            self.count = self.count.wrapping_add(1);
        }

        self.clock.previous = self.clock.current;
        Ok(())
    }

    fn simulation_time(&self) -> SimulationTime {
        self.time
    }

    fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
        if self.lifecycle == Lifecycle::Finalized {
            return Err(MockError::Finalized);
        }

        let Some(next_time) = self.time.checked_add(delta) else {
            return Err(MockError::TimeOverflow);
        };

        self.time = next_time;

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), Self::Error> {
        self.lifecycle = Lifecycle::Finalized;
        Ok(())
    }
}

impl TraceableDut for MockDut {
    fn open_trace(&mut self, _path: &std::path::Path) -> Result<(), Self::Error> {
        Ok(())
    }

    fn close_trace(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn trace_is_open(&self) -> bool {
        false
    }
}

impl TimedDut for MockDut {
    fn events_pending(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn next_time_slot(&self) -> Result<Option<SimulationTime>, Self::Error> {
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug, Default, Clock)]
#[vvm(dut = MockDut, clock = "clk")]
struct MockClock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Drive)]
#[vvm(dut = MockDut)]
#[allow(clippy::needless_pass_by_value)]
struct Stimulus {
    #[vvm(port)]
    reset_n: bool,
    #[vvm(port)]
    enable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Sample)]
#[vvm(dut = MockDut)]
struct Observation {
    #[vvm(port)]
    count: u8,
}

#[derive(Debug, Default)]
struct MockReferenceModel {
    count: u8,
}

impl ReferenceModel<Stimulus> for MockReferenceModel {
    type Expected = Observation;

    fn predict(&mut self, stimulus: &Stimulus) -> Self::Expected {
        if !stimulus.reset_n {
            self.count = 0;
        } else if stimulus.enable {
            self.count = self.count.wrapping_add(1);
        }

        Observation { count: self.count }
    }
}

#[test]
fn facade_exports_traits_derives_and_runner() {
    fn drive_once<D, S>(stimulus: &S, dut: &mut D)
    where
        D: vvm::dut::Dut,
        S: vvm::dut::Drive<D>,
    {
        let result = stimulus.drive(dut);
        assert!(matches!(result, Ok(())));
    }

    fn assert_traceable<D: vvm::dut::TraceableDut>(_dut: &D) {}
    fn accepts_timed_dut<D: TimedDut>(dut: &D) {
        let _ = dut.simulation_time();
    }

    let mut dut = MockDut::default();
    let stimulus = Stimulus {
        reset_n: false,
        enable: false,
    };

    drive_once(&stimulus, &mut dut);
    assert_traceable(&dut);
    accepts_timed_dut(&dut);

    assert_eq!(Dut::simulation_time(&dut), SimulationTime::ZERO);

    let advance = Dut::advance_time(&mut dut, TimeStep::ONE);

    assert!(matches!(advance, Ok(())));
    assert_eq!(Dut::simulation_time(&dut), SimulationTime::from_ticks(1));

    let result = Testbench::new(MockDut::default())
        .with_sequence([
            Stimulus {
                reset_n: false,
                enable: false,
            },
            Stimulus {
                reset_n: true,
                enable: true,
            },
        ])
        .with_reference_model(MockReferenceModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(MockClock)
        .run::<Observation>();

    assert!(result.passed());
    assert_eq!(result.failure_count(), 0);
    assert_eq!(result.cycles(), 2);
    assert_eq!(result.checks(), 2);
    assert_eq!(result.final_time(), SimulationTime::from_ticks(4));
    assert!(result.simulation_error().is_none());
    assert!(result.finalization_error().is_none());
}

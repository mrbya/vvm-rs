//! Public facade integration coverage.

use std::borrow::Borrow;

use vvm::TraceableDut;
use vvm::prelude::*;

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
        D: vvm::Dut,
        S: vvm::Drive<D>,
    {
        let result = stimulus.drive(dut);
        assert!(matches!(result, Ok(())));
    }

    fn assert_traceable<D: vvm::TraceableDut>(_dut: &D) {}

    let mut dut = MockDut::default();
    let stimulus = Stimulus {
        reset_n: false,
        enable: false,
    };

    drive_once(&stimulus, &mut dut);
    assert_traceable(&dut);

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

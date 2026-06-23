use thiserror::Error;
use vvm_core::{Clock, Mismatch, ReferenceModel, TestResult};
use vvm_macros::{Drive, Sample};

use crate::generated::{Counter, CounterError};

/// Counter simulation error.
#[derive(Debug, Error)]
pub enum Error {
    /// DUT access or lifecycle failure.
    #[error(transparent)]
    Dut(#[from] CounterError),

    /// Expected and observed counter value differed.
    #[error("counter verification failed")]
    TestFailed,
}

/// Counter simulation result.
pub type Result<T> = std::result::Result<T, Error>;

/// Counter scoreboard mismatch.
pub type CounterMismatch = Mismatch<CounterObservation, CounterObservation>;

/// Complete result of one counter verification run.
pub type CounterTestResult = TestResult<CounterStimulus, CounterMismatch, CounterError>;

/// Clock driver for the generated dounter DUT.
#[derive(Debug, Clone, Copy, Default)]
pub struct CounterClock;

impl Clock<Counter> for CounterClock {
    fn drive_inactive(
        &mut self,
        dut: &mut Counter,
    ) -> std::prelude::v1::Result<(), <Counter as vvm_core::Dut>::Error> {
        dut.set_clk(false)
    }

    fn drive_active(
        &mut self,
        dut: &mut Counter,
    ) -> std::prelude::v1::Result<(), <Counter as vvm_core::Dut>::Error> {
        dut.set_clk(true)
    }
}

/// Inputs applied during one counter cycle.
///
/// Clock control is intentionally excluded because the explicit simulation
/// loop owns active-edge timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Drive)]
#[vvm(dut = crate::generated::Counter)]
pub struct CounterStimulus {
    /// Active-low reset input.
    #[vvm(port)]
    reset_n: bool,

    /// Counter enable input.
    #[vvm(port)]
    enable: bool,
}

impl CounterStimulus {
    /// Create one counter stimulus.
    #[must_use]
    pub const fn new(reset_n: bool, enable: bool) -> Self {
        Self { reset_n, enable }
    }
}

/// Counter output sampled after an active edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sample)]
#[vvm(dut = crate::generated::Counter)]
pub struct CounterObservation {
    /// Sampled counter value.
    #[vvm(port)]
    count: u8,
}

/// Stateful behavioral model of the counter.
#[derive(Debug, Default)]
pub struct CounterReferenceModel {
    /// Expected current counter value.
    count: u8,
}

impl ReferenceModel<CounterStimulus> for CounterReferenceModel {
    type Expected = CounterObservation;

    fn predict(&mut self, stimulus: &CounterStimulus) -> Self::Expected {
        if !stimulus.reset_n {
            self.count = 0;
        } else if stimulus.enable {
            self.count = self.count.wrapping_add(1);
        }

        CounterObservation { count: self.count }
    }
}

/// Returns the deterministic counter stimulus sequence.
///
/// The sequence covers reset assertion, reset release, enabled counting,
/// disabled hold behavior, and reset reassertion.
pub fn counter_sequence() -> impl ExactSizeIterator<Item = CounterStimulus> {
    [
        CounterStimulus::new(false, false),
        CounterStimulus::new(true, false),
        CounterStimulus::new(true, true),
        CounterStimulus::new(true, true),
        CounterStimulus::new(true, true),
        CounterStimulus::new(true, false),
        CounterStimulus::new(false, false),
    ]
    .into_iter()
}

#[cfg(test)]
mod tests {
    use vvm_core::{ExactScoreboard, ReferenceModel, Scoreboard};

    use super::{CounterClock, CounterObservation, CounterReferenceModel, counter_sequence};
    use crate::generated::{Counter, Result};

    impl CounterObservation {
        /// Creates one counter observation.
        #[must_use]
        pub const fn new(count: u8) -> Self {
            Self { count }
        }

        /// Returns the sampled counter value.
        #[must_use]
        pub const fn count(self) -> u8 {
            self.count
        }
    }

    /// Deliberately incorrect counter model.
    #[derive(Debug, Default)]
    struct IncorrectCounterModel;

    impl vvm_core::ReferenceModel<crate::verification::CounterStimulus> for IncorrectCounterModel {
        type Expected = crate::verification::CounterObservation;

        fn predict(&mut self, _stimulus: &crate::verification::CounterStimulus) -> Self::Expected {
            crate::verification::CounterObservation::new(255)
        }
    }

    #[test]
    fn reference_model_matches_expected_sequence() {
        let mut model = CounterReferenceModel::default();

        let actual = counter_sequence()
            .map(|stimulus| model.predict(&stimulus).count())
            .collect::<Vec<_>>();

        assert_eq!(actual, vec![0, 0, 1, 2, 3, 3, 0]);
    }

    #[test]
    fn scoreboard_detects_intentional_mismatch() {
        let expected = CounterObservation { count: 3 };
        let observed = CounterObservation { count: 4 };
        let mut scoreboard = ExactScoreboard;

        assert!(scoreboard.check(expected, observed).is_err());
    }

    #[test]
    fn runner_reports_cycle_aware_mismatch() -> Result<()> {
        let dut = Counter::new()?;

        let result = vvm_core::Testbench::new(dut)
            .with_sequence(counter_sequence())
            .with_reference_model(IncorrectCounterModel)
            .with_scoreboard(vvm_core::ExactScoreboard)
            .with_clock(CounterClock)
            .run::<CounterObservation>();

        assert!(!result.passed());
        assert_eq!(result.failure_count(), 1);
        assert!(result.stopped_by_failure_policy());

        let failure = result.failures().first();

        assert_eq!(failure.map(vvm_core::CheckFailure::cycle), Some(0));

        Ok(())
    }
}

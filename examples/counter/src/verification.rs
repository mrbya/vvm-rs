use thiserror::Error;
use vvm_core::{Drive, Mismatch, ReferenceModel, Sample};

use crate::generated::{Counter, CounterError};

/// Counter simulation error.
#[derive(Debug, Error)]
pub enum Error {
    /// DUT access or lifecycle failure.
    #[error(transparent)]
    Dut(#[from] CounterError),

    /// Expected and observed counter value differed.
    #[error(transparent)]
    Scoreboard(#[from] CounterMismatch),
}

/// Counter simulation result.
pub type Result<T> = std::result::Result<T, Error>;

/// Counter scoreboard mismatch.
pub type CounterMismatch = Mismatch<CounterObservation, CounterObservation>;

/// Inputs applied during one counter cycle.
///
/// Clock control is intentionally excluded because the explicit simulation
/// loop owns active-edge timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterStimulus {
    /// Active-low reset input.
    reset_n: bool,

    /// Counter enable input.
    enable: bool,
}

impl CounterStimulus {
    /// Create one counter stimulus.
    #[must_use]
    pub const fn new(reset_n: bool, enable: bool) -> Self {
        Self { reset_n, enable }
    }
}

impl Drive<Counter> for CounterStimulus {
    fn drive(
        &self,
        dut: &mut Counter,
    ) -> std::prelude::v1::Result<(), <Counter as vvm_core::Dut>::Error> {
        dut.set_reset_n(self.reset_n)?;
        dut.set_enable(self.enable)
    }
}

/// Counter output sampled after an active edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterObservation {
    /// Sampled counter value.
    count: u8,
}

impl CounterObservation {
    /// Returns the sampled counter value.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.count
    }
}

impl Sample<Counter> for CounterObservation {
    fn sample(dut: &Counter) -> std::prelude::v1::Result<Self, <Counter as vvm_core::Dut>::Error> {
        Ok(Self {
            count: dut.count()?,
        })
    }
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

    use super::{CounterObservation, CounterReferenceModel, counter_sequence};

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
}

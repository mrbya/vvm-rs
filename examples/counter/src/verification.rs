use thiserror::Error;
use vvm::{
    Clock, Drive, Mismatch, RandomContext, ReferenceModel, ReplayToken, ReplayableSequence, Sample,
    Seed, TestResult,
};

use crate::counter::CounterError;

/// Counter simulation error.
#[derive(Debug, Error)]
pub enum Error {
    /// DUT access or lifecycle failure.
    #[error(transparent)]
    Dut(#[from] CounterError),

    /// Expected and observed counter value differed.
    #[error("counter verification failed")]
    TestFailed,

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Counter simulation result.
pub type Result<T> = std::result::Result<T, Error>;

/// Counter scoreboard mismatch.
pub type CounterMismatch = Mismatch<CounterObservation, CounterObservation>;

/// Complete result of one counter verification run.
pub type CounterTestResult = TestResult<CounterStimulus, CounterMismatch, CounterError>;

/// Clock driver for the generated dounter DUT.
#[derive(Debug, Clone, Copy, Default, Clock)]
#[vvm(
    dut = crate::counter::Counter,
    clock = "clk"
)]
pub struct CounterClock;

/// Inputs applied during one counter cycle.
///
/// Clock control is intentionally excluded because the explicit simulation
/// loop owns active-edge timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Drive)]
#[vvm(dut = crate::counter::Counter)]
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
#[vvm(dut = crate::counter::Counter)]
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
#[allow(dead_code)]
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

/// Replayable randomized counter sequence.
///
/// The first cycle always asserts reset. Remaining cycles use one random
/// 32-bit word each.
pub struct RandomCounterSequence {
    /// Deterministic random source.
    random: RandomContext,

    /// Number of stimuli not yet emitted.
    remaining: u64,

    /// Whether the mandatory reset preamble is pending.
    initial_reset: bool,
}

impl RandomCounterSequence {
    /// Creates a randomized sequence from a seed.
    #[must_use]
    pub fn new(seed: Seed, cycles: u64) -> Self {
        Self::from_replay(ReplayToken::new(seed), cycles)
    }

    /// Reconstructs a randomized sequence.
    #[must_use]
    pub fn from_replay(replay: ReplayToken, cycles: u64) -> Self {
        Self {
            random: RandomContext::from_replay(replay),
            remaining: cycles,
            initial_reset: cycles != 0,
        }
    }
}

impl ReplayableSequence for RandomCounterSequence {
    fn replay_token(&self) -> ReplayToken {
        self.random.replay_token()
    }
}

impl Iterator for RandomCounterSequence {
    type Item = CounterStimulus;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.remaining.checked_sub(1)?;

        self.remaining = remaining;

        if self.initial_reset {
            self.initial_reset = false;

            return Some(CounterStimulus::new(false, false));
        }

        let bits = self.random.next_u32();

        // Assert reset approximately one cycle in
        // sixteen. No range distribution is involved:
        // this mapping is part of the sequence contract.
        let reset_n = bits & 0x0f != 0;

        // Use a separate bit from the same random word.
        let enable = bits & 0x10 != 0;

        Some(CounterStimulus::new(reset_n, enable))
    }
}

#[cfg(test)]
mod tests {
    use vvm::{CheckFailure, ExactScoreboard, ReferenceModel, Scoreboard, Testbench};

    use super::{CounterClock, CounterObservation, CounterReferenceModel, counter_sequence};
    use crate::counter::{Counter, Result};

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

    impl vvm::ReferenceModel<crate::verification::CounterStimulus> for IncorrectCounterModel {
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

        let result = Testbench::new(dut)
            .with_sequence(counter_sequence())
            .with_reference_model(IncorrectCounterModel)
            .with_scoreboard(ExactScoreboard)
            .with_clock(CounterClock)
            .run::<CounterObservation>();

        assert!(!result.passed());
        assert_eq!(result.failure_count(), 1);
        assert!(result.stopped_by_failure_policy());

        let failure = result.failures().first();

        assert_eq!(failure.map(CheckFailure::cycle), Some(0));

        let report = result.detailed_report().to_string();

        assert!(report.contains("FAIL: 1 cycle, 1 check, 1 check failure",));

        assert!(report.contains("Check failures (1):"));

        assert!(report.contains("cycle 0 at 1 ticks"));

        assert!(report.contains("CounterStimulus"));

        assert!(report.contains("scoreboard mismatch"));

        Ok(())
    }
}

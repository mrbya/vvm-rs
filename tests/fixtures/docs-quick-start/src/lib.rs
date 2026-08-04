//! Independent quick-start fixture for the user guide.

// ANCHOR: include-dut
#[cfg(test)]
vvm::include_dut!(event_counter);
// ANCHOR_END: include-dut

#[cfg(test)]
mod tests {
    use vvm::random::{RandomContext, ReplayToken, ReplayableSequence, Seed};
    use vvm::coverage::{Bin, BuildError, Coverpoint, Cross2};
    use vvm::test::{TestContext, TestRunConfig};
    use vvm::testbench::{
        ExactScoreboard, Mismatch, ObservedCycle, ReferenceModel, Scoreboard, TestResult,
        Testbench,
    };
    use vvm::{Clock, Drive, Sample};

    use crate::event_counter::EventCounter;

    pub type EventCounterMismatch = Mismatch<EventCounterObservation, EventCounterObservation>;
    pub type EventCounterTestResult =
        TestResult<EventCounterStimulus, EventCounterMismatch, crate::event_counter::EventCounterError>;
    pub type Result<T> = std::result::Result<T, crate::event_counter::EventCounterError>;

    const RANDOM_CYCLES: u64 = 256;
    const DEFAULT_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x0bad_5eed_1234_5678));

    // ANCHOR: stimulus
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Drive)]
    #[vvm(dut = crate::event_counter::EventCounter)]
    struct EventCounterStimulus {
        #[vvm(port)]
        reset_n: bool,

        #[vvm(port = "event_pulse")]
        event: bool,
    }
    // ANCHOR_END: stimulus

    impl EventCounterStimulus {
        const fn new(reset_n: bool, event: bool) -> Self {
            Self { reset_n, event }
        }
    }

    // ANCHOR: observation
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Sample)]
    #[vvm(dut = crate::event_counter::EventCounter)]
    struct EventCounterObservation {
        #[vvm(port)]
        total: u8,
    }
    // ANCHOR_END: observation

    impl EventCounterObservation {
        const fn new(total: u8) -> Self {
            Self { total }
        }
    }

    // ANCHOR: clock
    #[derive(Debug, Clone, Copy, Default, Clock)]
    #[vvm(dut = crate::event_counter::EventCounter, clock = "clk")]
    struct EventCounterClock;
    // ANCHOR_END: clock

    // ANCHOR: sequence
    fn event_counter_sequence() -> impl ExactSizeIterator<Item = EventCounterStimulus> {
        [
            EventCounterStimulus::new(false, false),
            EventCounterStimulus::new(true, false),
            EventCounterStimulus::new(true, true),
            EventCounterStimulus::new(true, false),
            EventCounterStimulus::new(true, true),
            EventCounterStimulus::new(true, true),
            EventCounterStimulus::new(false, false),
            EventCounterStimulus::new(true, false),
        ]
        .into_iter()
    }
    // ANCHOR_END: sequence

    // ANCHOR: replayable-sequence
    struct RandomEventSequence {
        random: RandomContext,
        remaining: u64,
        initial_reset: bool,
    }

    impl RandomEventSequence {
        fn new(replay: ReplayToken, cycles: u64) -> Self {
            Self {
                random: RandomContext::from_replay(replay),
                remaining: cycles,
                initial_reset: cycles != 0,
            }
        }
    }

    impl ReplayableSequence for RandomEventSequence {
        fn replay_token(&self) -> ReplayToken {
            self.random.replay_token()
        }
    }

    impl Iterator for RandomEventSequence {
        type Item = EventCounterStimulus;

        fn next(&mut self) -> Option<Self::Item> {
            let remaining = self.remaining.checked_sub(1)?;

            self.remaining = remaining;

            if self.initial_reset {
                self.initial_reset = false;

                return Some(EventCounterStimulus::new(false, false));
            }

            let bits = self.random.next_u32();
            let reset_n = bits & 0x0f != 0;
            let event = bits & 0x10 != 0;

            Some(EventCounterStimulus::new(reset_n, event))
        }
    }
    // ANCHOR_END: replayable-sequence

    // ANCHOR: reference-model
    #[derive(Debug, Default)]
    struct EventCounterReferenceModel {
        total: u8,
    }

    impl ReferenceModel<EventCounterStimulus> for EventCounterReferenceModel {
        type Expected = EventCounterObservation;

        fn predict(&mut self, stimulus: &EventCounterStimulus) -> Self::Expected {
            if !stimulus.reset_n {
                self.total = 0;
            } else if stimulus.event {
                self.total = self.total.wrapping_add(1);
            }

            EventCounterObservation::new(self.total)
        }
    }
    // ANCHOR_END: reference-model

    /// Semantic activity observed during one sampled event-counter cycle.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum EventCounterActivity {
        Reset,
        Idle,
        Event,
        Invalid,
    }

    // ANCHOR: coverage-model
    #[derive(vvm::Coverage)]
    #[vvm(
        definition = "event_counter_coverage",
        revision = 1,
        stimulus = EventCounterStimulus,
        observation = EventCounterObservation,
    )]
    struct EventCounterCoverage {
        #[vvm(coverpoint(build = activity_coverpoint, sample = activity_for))]
        activity: Coverpoint<EventCounterActivity>,

        #[vvm(coverpoint(build = total_coverpoint, sample = total_for))]
        total_region: Coverpoint<u8>,

        #[vvm(cross(left = activity, right = total_region))]
        activity_x_total: Cross2,
    }

    fn activity_coverpoint(
        name: &'static str,
    ) -> std::result::Result<Coverpoint<EventCounterActivity>, BuildError> {
        Coverpoint::builder(name)
            .bin(Bin::value("idle", EventCounterActivity::Idle))
            .bin(Bin::value("event", EventCounterActivity::Event))
            .ignore_bin(Bin::value("reset", EventCounterActivity::Reset))
            .illegal_bin(Bin::value("invalid", EventCounterActivity::Invalid))
            .build()
    }

    fn total_coverpoint(name: &'static str) -> std::result::Result<Coverpoint<u8>, BuildError> {
        Coverpoint::builder(name)
            .bin(Bin::value("zero", 0_u8))
            .bin(Bin::inclusive_range("small", 1_u8, 3_u8))
            .bin(Bin::inclusive_range("large", 4_u8, u8::MAX))
            .build()
    }

    const fn activity_for(
        cycle: ObservedCycle<'_, EventCounterStimulus, EventCounterObservation>,
    ) -> EventCounterActivity {
        if !cycle.stimulus().reset_n {
            EventCounterActivity::Reset
        } else if cycle.stimulus().event {
            EventCounterActivity::Event
        } else {
            EventCounterActivity::Idle
        }
    }

    const fn total_for(cycle: ObservedCycle<'_, EventCounterStimulus, EventCounterObservation>) -> u8 {
        cycle.observed().total
    }
    // ANCHOR_END: coverage-model

    // ANCHOR: scoreboard
    #[test]
    fn exact_scoreboard_compares_expected_and_observed_values() {
        let mut scoreboard = ExactScoreboard;
        let expected = EventCounterObservation::new(3);
        let observed = EventCounterObservation::new(2);
        let mismatch = scoreboard.check(&expected, &observed).unwrap_err();

        assert_eq!(**mismatch.expected(), expected);
        assert_eq!(**mismatch.observed(), observed);
    }
    // ANCHOR_END: scoreboard

    // ANCHOR: coverage-test
    /// Covered event-counter smoke test.
    #[vvm::test(trace, coverage)]
    fn event_counter_coverage(
        context: &mut TestContext,
    ) -> std::result::Result<EventCounterTestResult, Box<dyn std::error::Error>> {
        let mut dut = EventCounter::new()?;

        context.config().configure_trace(&mut dut)?;

        Ok(Testbench::new(dut)
            .with_sequence(event_counter_sequence())
            .with_reference_model(EventCounterReferenceModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(EventCounterClock)
            .with_coverage(EventCounterCoverage::new("dut.event_counter")?)
            .run_covered::<EventCounterObservation>(context))
    }
    // ANCHOR_END: coverage-test

    // ANCHOR: test
    /// Deterministic event-counter smoke test.
    #[vvm::test(trace)]
    fn event_counter_smoke(config: &TestRunConfig) -> Result<EventCounterTestResult> {
        let mut dut = EventCounter::new()?;

        config.configure_trace(&mut dut)?;

        let result = Testbench::new(dut)
            .with_sequence(event_counter_sequence())
            .with_reference_model(EventCounterReferenceModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(EventCounterClock)
            .run::<EventCounterObservation>();

        Ok(result)
    }
    // ANCHOR_END: test

    // ANCHOR: replay-test
    /// Replayable randomized event-counter regression.
    #[vvm::test(trace, cycles, replay(default = DEFAULT_REPLAY))]
    fn event_counter_random(config: &TestRunConfig) -> Result<EventCounterTestResult> {
        let mut dut = EventCounter::new()?;
        let sequence = RandomEventSequence::new(
            config.replay_token_or(DEFAULT_REPLAY),
            config.cycles_or(RANDOM_CYCLES),
        );

        config.configure_trace(&mut dut)?;

        let result = Testbench::new(dut)
            .with_replayable_sequence(sequence)
            .with_reference_model(EventCounterReferenceModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(EventCounterClock)
            .run::<EventCounterObservation>();

        Ok(result)
    }
    // ANCHOR_END: replay-test

    #[test]
    fn sequence_matches_expected_model_values() {
        let mut model = EventCounterReferenceModel::default();

        let actual = event_counter_sequence()
            .map(|stimulus| model.predict(&stimulus).total)
            .collect::<Vec<_>>();

        assert_eq!(actual, vec![0, 0, 1, 1, 2, 3, 0, 0]);
    }

    // ANCHOR: coverage-artifact
    #[test]
    fn covered_run_exposes_a_coverage_snapshot(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let run = __vvm_test_descriptor_event_counter_coverage.run(&TestRunConfig::new())?;
        let coverage = run.coverage().ok_or("missing coverage snapshot")?;

        assert_eq!(coverage.groups().len(), 1);
        assert!(coverage.group("dut.event_counter").is_some());

        Ok(())
    }
    // ANCHOR_END: coverage-artifact
}

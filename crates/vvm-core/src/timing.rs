//! Scheduler for simulator-owned delayed timing events.
//!
//! [`ClockScheduler`](crate::ClockScheduler) schedules Rust-driven input-clock
//! transitions, while [`Testbench`](crate::Testbench) runs cycle-driven
//! stimulus, sampling, prediction, and checking. This scheduler instead
//! processes absolute delayed-event times reported by a [`TimedDut`].

use std::num::NonZeroU64;

use crate::{Dut, SimulationTime, TimeStep, TimedDut};

/// Operation performed by a timing scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimingStage {
    /// Initial DUT evaluation.
    Initialize,
    /// Querying whether delayed events remain.
    EventsPending,
    /// Querying the next absolute delayed-event time.
    NextTimeSlot,
    /// Advancing simulation time.
    AdvanceTime,
    /// Evaluating a delayed time slot.
    EvaluateTimeSlot,
}

impl std::fmt::Display for TimingStage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match *self {
            Self::Initialize => "initializing the timing-enabled DUT",
            Self::EventsPending => "querying pending timing events",
            Self::NextTimeSlot => "querying the next timing event",
            Self::AdvanceTime => "advancing to the next timing event",
            Self::EvaluateTimeSlot => "evaluating the current timing event",
        })
    }
}

/// Error returned by [`TimingScheduler`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TimingSchedulerError<E> {
    /// Initialization was requested more than once.
    #[error("timing scheduler has already been initialized")]
    AlreadyInitialized,
    /// An operation requiring initialization was called too early.
    #[error("timing scheduler has not been initialized")]
    NotInitialized,
    /// A DUT operation failed.
    #[error("timing scheduler failed while {stage} at {time}: {source}")]
    Dut {
        /// Scheduler stage.
        stage: TimingStage,
        /// DUT time associated with the failure.
        time: SimulationTime,
        /// Underlying DUT error.
        #[source]
        source: E,
    },
    /// The DUT reported pending work but no next time.
    #[error("timed DUT reported a pending event but no next time slot at {time}")]
    MissingTimeSlot {
        /// Current DUT time.
        time: SimulationTime,
    },
    /// The DUT reported an event earlier than the current time.
    #[error("timed DUT reported event time {next} before current time {current}")]
    TimeSlotInPast {
        /// Current DUT time.
        current: SimulationTime,
        /// Invalid reported time.
        next: SimulationTime,
    },
    /// The DUT reported the current time as the next slot.
    #[error("timed DUT reported non-advancing event time {time}")]
    NonAdvancingTimeSlot {
        /// Current and reported time.
        time: SimulationTime,
    },
    /// The configured total delayed-slot limit was reached.
    #[error("timing scheduler reached its {limit}-slot limit at {time} while events remained")]
    TimeSlotLimitReached {
        /// Maximum total number of delayed slots.
        limit: u64,
        /// DUT time when processing stopped.
        time: SimulationTime,
    },
    /// Scheduler counters could not be incremented safely.
    #[error("timing scheduler statistics overflowed at {time}")]
    StatisticsOverflow {
        /// DUT time associated with overflow.
        time: SimulationTime,
    },
}

impl<E> TimingSchedulerError<E> {
    /// Returns the failed DUT stage.
    #[must_use]
    pub const fn stage(&self) -> Option<TimingStage> {
        match *self {
            Self::Dut { stage, .. } => Some(stage),
            _ => None,
        }
    }

    /// Returns the underlying DUT error.
    #[must_use]
    pub const fn source_error(&self) -> Option<&E> {
        match *self {
            Self::Dut { ref source, .. } => Some(source),
            _ => None,
        }
    }

    /// Returns the relevant scheduler time when the error has one.
    #[must_use]
    pub const fn time(&self) -> Option<SimulationTime> {
        match *self {
            Self::Dut { time, .. }
            | Self::MissingTimeSlot { time }
            | Self::NonAdvancingTimeSlot { time }
            | Self::TimeSlotLimitReached { time, .. }
            | Self::StatisticsOverflow { time } => Some(time),
            Self::TimeSlotInPast { current, .. } => Some(current),
            Self::AlreadyInitialized | Self::NotInitialized => None,
        }
    }
}

/// One successfully evaluated delayed-event time slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingEvent {
    /// Zero-based delayed-slot ordinal.
    ordinal: u64,
    /// Absolute evaluated simulation time.
    time: SimulationTime,
    /// Positive elapsed time since the preceding evaluation.
    elapsed: TimeStep,
}

impl TimingEvent {
    /// Creates timing-event metadata.
    const fn new(ordinal: u64, time: SimulationTime, elapsed: TimeStep) -> Self {
        Self {
            ordinal,
            time,
            elapsed,
        }
    }

    /// Returns the zero-based delayed-slot ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }

    /// Returns the absolute event time.
    #[must_use]
    pub const fn time(self) -> SimulationTime {
        self.time
    }

    /// Returns time elapsed since the preceding evaluation.
    #[must_use]
    pub const fn elapsed(self) -> TimeStep {
        self.elapsed
    }
}

/// Summary of bounded delayed-event execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingRun {
    /// DUT time recorded during initialization.
    start_time: SimulationTime,
    /// DUT time when execution became idle.
    final_time: SimulationTime,
    /// Successfully processed delayed slots.
    time_slots: u64,
    /// Successfully completed evaluations, including initialization.
    evaluations: u64,
}

impl TimingRun {
    /// Returns scheduler initialization time.
    #[must_use]
    pub const fn start_time(self) -> SimulationTime {
        self.start_time
    }

    /// Returns time when the delayed-event queue became idle.
    #[must_use]
    pub const fn final_time(self) -> SimulationTime {
        self.final_time
    }

    /// Returns successfully processed delayed slots.
    #[must_use]
    pub const fn time_slots(self) -> u64 {
        self.time_slots
    }

    /// Returns total successful evaluations.
    #[must_use]
    pub const fn evaluations(self) -> u64 {
        self.evaluations
    }
}

/// Lifecycle state of a timing scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimingSchedulerState {
    /// Initial evaluation has not occurred.
    Uninitialized,
    /// Initial evaluation completed.
    Initialized {
        /// DUT time at initialization.
        start_time: SimulationTime,
    },
}

/// Processes delayed-event slots reported by a [`TimedDut`].
///
/// Initialization evaluates once at the current DUT time, but that evaluation
/// is not a delayed slot. For a finite delayed process, the execution timeline
/// can be an initial evaluation at t0 followed by delayed slots at t2, t5, and
/// t10. That execution reports three `time_slots` and four `evaluations`.
///
/// Delayed event times are absolute; each is converted to a positive relative
/// [`TimeStep`] for [`Dut::advance_time`]. Equal-time and past event slots are
/// unsupported and rejected. Open waveform tracing before initialization when
/// time-zero dumping is required. This scheduler does not own tracing or
/// finalize the DUT, so callers must finalize it explicitly after execution.
///
/// For manual stepping:
/// ```ignore
/// let mut scheduler = TimingScheduler::new();
/// scheduler.initialize(&mut dut)?;
/// while let Some(event) = scheduler.advance_next(&mut dut)? {
///     println!("processed slot {} at {}", event.ordinal(), event.time());
/// }
/// ```
///
/// For bounded execution, use `scheduler.run_until_idle(&mut dut,
/// max_time_slots)?`. The limit covers the scheduler's lifetime total, and
/// this scheduler never finalizes the DUT. Timing execution is separate from
/// cycle-driven [`crate::Testbench`] execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimingScheduler {
    /// Scheduler lifecycle.
    state: TimingSchedulerState,
    /// Successfully processed delayed-event slots.
    time_slots: u64,
    /// Successfully completed evaluations, including initialization.
    evaluations: u64,
}

impl TimingScheduler {
    /// Creates an uninitialized scheduler.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: TimingSchedulerState::Uninitialized,
            time_slots: 0,
            evaluations: 0,
        }
    }

    /// Returns whether initial evaluation completed.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        matches!(self.state, TimingSchedulerState::Initialized { .. })
    }

    /// Returns the DUT time recorded during initialization.
    #[must_use]
    pub const fn start_time(&self) -> Option<SimulationTime> {
        match self.state {
            TimingSchedulerState::Uninitialized => None,
            TimingSchedulerState::Initialized { start_time } => Some(start_time),
        }
    }

    /// Returns the number of successfully processed delayed slots.
    #[must_use]
    pub const fn time_slots(&self) -> u64 {
        self.time_slots
    }

    /// Returns the number of successfully completed DUT evaluations.
    #[must_use]
    pub const fn evaluations(&self) -> u64 {
        self.evaluations
    }

    /// Initializes delayed-event execution.
    ///
    /// Initialization evaluates the DUT exactly once at its current simulation
    /// time. It does not query timing events, advance time, or finalize.
    ///
    /// # Errors
    ///
    /// Returns an error when initialization already completed or the DUT cannot
    /// be evaluated.
    pub fn initialize<D>(&mut self, dut: &mut D) -> Result<(), TimingSchedulerError<D::Error>>
    where
        D: TimedDut,
    {
        if self.is_initialized() {
            return Err(TimingSchedulerError::AlreadyInitialized);
        }

        let start_time = dut.simulation_time();

        Dut::evaluate(dut).map_err(|source| TimingSchedulerError::Dut {
            stage: TimingStage::Initialize,
            time: start_time,
            source,
        })?;

        self.state = TimingSchedulerState::Initialized { start_time };
        self.evaluations = 1;

        Ok(())
    }

    /// Advances to and evaluates one delayed-event time slot.
    ///
    /// Returns `Ok(None)` when no delayed events remain. Query failures and
    /// invalid times leave the DUT and statistics unchanged. An advance failure
    /// leaves statistics unchanged; an evaluation failure may leave the DUT at
    /// the target time, is not rolled back, and does not record the slot.
    ///
    /// # Errors
    ///
    /// Returns an error when uninitialized, a DUT operation fails, or a
    /// reported event time violates scheduler invariants.
    pub fn advance_next<D>(
        &mut self,
        dut: &mut D,
    ) -> Result<Option<TimingEvent>, TimingSchedulerError<D::Error>>
    where
        D: TimedDut,
    {
        if !self.is_initialized() {
            return Err(TimingSchedulerError::NotInitialized);
        }

        let current = dut.simulation_time();
        let pending = dut
            .events_pending()
            .map_err(|source| TimingSchedulerError::Dut {
                stage: TimingStage::EventsPending,
                time: current,
                source,
            })?;

        if !pending {
            return Ok(None);
        }

        let next = dut
            .next_time_slot()
            .map_err(|source| TimingSchedulerError::Dut {
                stage: TimingStage::NextTimeSlot,
                time: current,
                source,
            })?
            .ok_or(TimingSchedulerError::MissingTimeSlot { time: current })?;

        let elapsed = event_delta::<D::Error>(current, next)?;
        let (next_time_slots, next_evaluations) = self.next_statistics::<D::Error>(current)?;

        Dut::advance_time(dut, elapsed).map_err(|source| TimingSchedulerError::Dut {
            stage: TimingStage::AdvanceTime,
            time: current,
            source,
        })?;

        Dut::evaluate(dut).map_err(|source| TimingSchedulerError::Dut {
            stage: TimingStage::EvaluateTimeSlot,
            time: next,
            source,
        })?;

        let event = TimingEvent::new(self.time_slots, next, elapsed);

        self.time_slots = next_time_slots;
        self.evaluations = next_evaluations;

        Ok(Some(event))
    }

    /// Processes delayed-event slots until the DUT becomes idle.
    ///
    /// An uninitialized scheduler is initialized automatically. The nonzero
    /// limit applies to all slots processed over this scheduler's lifetime;
    /// callers resuming it must select a limit at least as large as
    /// [`Self::time_slots`] when they expect further processing. This method
    /// does not finalize the DUT.
    ///
    /// # Errors
    ///
    /// Returns an error when a DUT operation fails, a reported event time is
    /// invalid, or events remain after the total slot limit is reached.
    pub fn run_until_idle<D>(
        &mut self,
        dut: &mut D,
        max_time_slots: NonZeroU64,
    ) -> Result<TimingRun, TimingSchedulerError<D::Error>>
    where
        D: TimedDut,
    {
        if !self.is_initialized() {
            self.initialize(dut)?;
        }

        loop {
            if self.time_slots >= max_time_slots.get() {
                let current = dut.simulation_time();
                let pending = dut
                    .events_pending()
                    .map_err(|source| TimingSchedulerError::Dut {
                        stage: TimingStage::EventsPending,
                        time: current,
                        source,
                    })?;

                if pending {
                    return Err(TimingSchedulerError::TimeSlotLimitReached {
                        limit: max_time_slots.get(),
                        time: current,
                    });
                }

                return self.run_summary(dut);
            }

            if self.advance_next(dut)?.is_none() {
                return self.run_summary(dut);
            }
        }
    }

    /// Calculates statistics before a delayed slot can mutate the DUT.
    const fn next_statistics<E>(
        &self,
        time: SimulationTime,
    ) -> Result<(u64, u64), TimingSchedulerError<E>> {
        let Some(time_slots) = self.time_slots.checked_add(1) else {
            return Err(TimingSchedulerError::StatisticsOverflow { time });
        };

        let Some(evaluations) = self.evaluations.checked_add(1) else {
            return Err(TimingSchedulerError::StatisticsOverflow { time });
        };

        Ok((time_slots, evaluations))
    }

    /// Builds an idle-run summary from initialized scheduler state.
    fn run_summary<D>(&self, dut: &D) -> Result<TimingRun, TimingSchedulerError<D::Error>>
    where
        D: TimedDut,
    {
        let start_time = match self.state {
            TimingSchedulerState::Uninitialized => {
                return Err(TimingSchedulerError::NotInitialized);
            }
            TimingSchedulerState::Initialized { start_time } => start_time,
        };

        Ok(TimingRun {
            start_time,
            final_time: dut.simulation_time(),
            time_slots: self.time_slots,
            evaluations: self.evaluations,
        })
    }
}

impl Default for TimingScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts an absolute delayed-event time into a positive relative step.
fn event_delta<E>(
    current: SimulationTime,
    next: SimulationTime,
) -> Result<TimeStep, TimingSchedulerError<E>> {
    if next < current {
        return Err(TimingSchedulerError::TimeSlotInPast { current, next });
    }

    if next == current {
        return Err(TimingSchedulerError::NonAdvancingTimeSlot { time: current });
    }

    let Some(delta_ticks) = next.ticks().checked_sub(current.ticks()) else {
        return Err(TimingSchedulerError::TimeSlotInPast { current, next });
    };

    let Some(delta_ticks) = NonZeroU64::new(delta_ticks) else {
        return Err(TimingSchedulerError::NonAdvancingTimeSlot { time: current });
    };

    Ok(TimeStep::from_nonzero(delta_ticks))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::VecDeque;

    use super::{TimingScheduler, TimingSchedulerError, TimingStage};
    use crate::{Dut, SimulationTime, TimeStep, TimedDut};

    /// Failure injected into one mock DUT operation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MockFailure {
        Initialize,
        Pending,
        Next,
        Advance,
        SlotEvaluate,
    }

    impl std::fmt::Display for MockFailure {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("mock timing failure")
        }
    }

    impl std::error::Error for MockFailure {}

    /// Pure-Rust timed DUT whose queue is consumed only by evaluation.
    struct MockTimedDut {
        time: SimulationTime,
        events: VecDeque<SimulationTime>,
        evaluations: Vec<SimulationTime>,
        advances: u64,
        finalizations: u64,
        failure: Option<MockFailure>,
        pending_queries: Cell<u64>,
        next_queries: Cell<u64>,
        pending_override: Option<bool>,
    }

    impl MockTimedDut {
        fn new(events: impl IntoIterator<Item = u64>) -> Self {
            Self {
                time: SimulationTime::ZERO,
                events: events.into_iter().map(SimulationTime::from_ticks).collect(),
                evaluations: Vec::new(),
                advances: 0,
                finalizations: 0,
                failure: None,
                pending_queries: Cell::new(0),
                next_queries: Cell::new(0),
                pending_override: None,
            }
        }

        fn failing(events: impl IntoIterator<Item = u64>, failure: MockFailure) -> Self {
            let mut dut = Self::new(events);

            dut.failure = Some(failure);

            dut
        }

        fn fails(&self, failure: MockFailure) -> bool {
            self.failure == Some(failure)
        }
    }

    impl Dut for MockTimedDut {
        type Error = MockFailure;

        fn evaluate(&mut self) -> Result<(), Self::Error> {
            let failure = if self.evaluations.is_empty() {
                MockFailure::Initialize
            } else {
                MockFailure::SlotEvaluate
            };

            if self.fails(failure) {
                return Err(failure);
            }

            self.evaluations.push(self.time);

            if self.events.front() == Some(&self.time) {
                let _ = self.events.pop_front();
            }

            Ok(())
        }

        fn simulation_time(&self) -> SimulationTime {
            self.time
        }

        fn advance_time(&mut self, delta: TimeStep) -> Result<(), Self::Error> {
            if self.fails(MockFailure::Advance) {
                return Err(MockFailure::Advance);
            }

            let Some(next) = self.time.checked_add(delta) else {
                return Err(MockFailure::Advance);
            };

            self.time = next;
            self.advances = self.advances.saturating_add(1);

            Ok(())
        }

        fn finalize(&mut self) -> Result<(), Self::Error> {
            self.finalizations = self.finalizations.saturating_add(1);

            Ok(())
        }
    }

    impl TimedDut for MockTimedDut {
        fn events_pending(&self) -> Result<bool, Self::Error> {
            self.pending_queries
                .set(self.pending_queries.get().saturating_add(1));

            if self.fails(MockFailure::Pending) {
                return Err(MockFailure::Pending);
            }

            Ok(self.pending_override.unwrap_or(!self.events.is_empty()))
        }

        fn next_time_slot(&self) -> Result<Option<SimulationTime>, Self::Error> {
            self.next_queries
                .set(self.next_queries.get().saturating_add(1));

            if self.fails(MockFailure::Next) {
                return Err(MockFailure::Next);
            }

            Ok(self.events.front().copied())
        }
    }

    fn limit(value: u64) -> std::num::NonZeroU64 {
        std::num::NonZeroU64::new(value).unwrap_or(std::num::NonZeroU64::MIN)
    }

    #[test]
    fn construction_and_initialization_are_isolated() -> Result<(), MockFailure> {
        let mut dut = MockTimedDut::new([2]);
        let mut scheduler = TimingScheduler::new();

        assert!(!scheduler.is_initialized());
        assert_eq!(scheduler, TimingScheduler::default());
        assert_eq!(dut.evaluations.len(), 0);
        assert_eq!(dut.pending_queries.get(), 0);

        scheduler.initialize(&mut dut).map_err(|error| {
            error
                .source_error()
                .copied()
                .unwrap_or(MockFailure::Initialize)
        })?;

        assert!(scheduler.is_initialized());
        assert_eq!(scheduler.start_time(), Some(SimulationTime::ZERO));
        assert_eq!(scheduler.time_slots(), 0);
        assert_eq!(scheduler.evaluations(), 1);
        assert_eq!(dut.evaluations, vec![SimulationTime::ZERO]);
        assert_eq!(dut.advances, 0);
        assert_eq!(dut.pending_queries.get(), 0);
        assert_eq!(dut.next_queries.get(), 0);

        Ok(())
    }

    #[test]
    fn steps_absolute_events_with_relative_deltas() -> Result<(), TimingSchedulerError<MockFailure>>
    {
        let mut dut = MockTimedDut::new([2, 5, 10]);
        let mut scheduler = TimingScheduler::new();

        scheduler.initialize(&mut dut)?;

        for (ordinal, time, elapsed) in [(0, 2, 2), (1, 5, 3), (2, 10, 5)] {
            let event = scheduler
                .advance_next(&mut dut)?
                .ok_or(TimingSchedulerError::NotInitialized)?;
            assert_eq!(event.ordinal(), ordinal);
            assert_eq!(event.time(), SimulationTime::from_ticks(time));
            assert_eq!(event.elapsed().ticks(), elapsed);
        }

        assert_eq!(scheduler.advance_next(&mut dut)?, None);
        assert_eq!(
            dut.evaluations,
            [0, 2, 5, 10].map(SimulationTime::from_ticks)
        );
        assert_eq!(scheduler.time_slots(), 3);
        assert_eq!(scheduler.evaluations(), 4);

        Ok(())
    }

    #[test]
    fn rejects_invalid_event_times_without_mutation()
    -> Result<(), TimingSchedulerError<MockFailure>> {
        for next in [3, 5] {
            let mut dut = MockTimedDut::new([]);
            dut.time = SimulationTime::from_ticks(5);
            let mut scheduler = TimingScheduler::new();

            scheduler.initialize(&mut dut)?;
            dut.events.push_back(SimulationTime::from_ticks(next));

            let result = scheduler.advance_next(&mut dut);

            assert!(matches!(
                result,
                Err(TimingSchedulerError::TimeSlotInPast { .. }
                    | TimingSchedulerError::NonAdvancingTimeSlot { .. })
            ));
            assert_eq!(dut.time, SimulationTime::from_ticks(5));
            assert_eq!(dut.evaluations.len(), 1);
            assert_eq!(scheduler.time_slots(), 0);
            assert_eq!(scheduler.evaluations(), 1);
        }

        Ok(())
    }

    #[test]
    fn failure_stages_and_statistics_are_preserved() -> Result<(), TimingSchedulerError<MockFailure>>
    {
        let cases = [
            (
                MockFailure::Pending,
                TimingStage::EventsPending,
                SimulationTime::ZERO,
            ),
            (
                MockFailure::Next,
                TimingStage::NextTimeSlot,
                SimulationTime::ZERO,
            ),
            (
                MockFailure::Advance,
                TimingStage::AdvanceTime,
                SimulationTime::ZERO,
            ),
            (
                MockFailure::SlotEvaluate,
                TimingStage::EvaluateTimeSlot,
                SimulationTime::from_ticks(2),
            ),
        ];

        for (failure, stage, time) in cases {
            let mut dut = MockTimedDut::failing([2], failure);
            let mut scheduler = TimingScheduler::new();

            scheduler.initialize(&mut dut)?;

            let error = scheduler
                .advance_next(&mut dut)
                .expect_err("injected failure must propagate");

            assert_eq!(error.stage(), Some(stage));
            assert_eq!(error.source_error(), Some(&failure));
            assert!(
                matches!(error, TimingSchedulerError::Dut { time: failure_time, .. } if failure_time == time)
            );
            assert_eq!(scheduler.time_slots(), 0);
            assert_eq!(scheduler.evaluations(), 1);
        }

        Ok(())
    }

    #[test]
    fn bounded_run_uses_lifetime_limit_and_never_finalizes()
    -> Result<(), TimingSchedulerError<MockFailure>> {
        let mut dut = MockTimedDut::new([2, 4, 6, 8]);
        let mut scheduler = TimingScheduler::new();

        scheduler.initialize(&mut dut)?;
        let _ = scheduler.advance_next(&mut dut)?;

        let error = scheduler
            .run_until_idle(&mut dut, limit(3))
            .expect_err("fourth event exceeds limit");

        assert_eq!(
            error,
            TimingSchedulerError::TimeSlotLimitReached {
                limit: 3,
                time: SimulationTime::from_ticks(6)
            }
        );
        assert_eq!(scheduler.time_slots(), 3);
        assert_eq!(scheduler.evaluations(), 4);
        assert_eq!(dut.events.front(), Some(&SimulationTime::from_ticks(8)));
        assert_eq!(dut.finalizations, 0);

        assert_eq!(Dut::finalize(&mut dut), Ok(()));
        assert_eq!(dut.finalizations, 1);

        Ok(())
    }

    #[test]
    fn bounded_run_summarizes_exact_limit_idle_queue()
    -> Result<(), TimingSchedulerError<MockFailure>> {
        let mut dut = MockTimedDut::new([2, 5, 10]);
        let mut scheduler = TimingScheduler::new();

        let run = scheduler.run_until_idle(&mut dut, limit(3))?;

        assert_eq!(run.start_time(), SimulationTime::ZERO);
        assert_eq!(run.final_time(), SimulationTime::from_ticks(10));
        assert_eq!(run.time_slots(), 3);
        assert_eq!(run.evaluations(), 4);
        assert_eq!(dut.finalizations, 0);

        Ok(())
    }

    #[test]
    fn detects_counter_overflow_before_touching_dut()
    -> Result<(), TimingSchedulerError<MockFailure>> {
        let mut dut = MockTimedDut::new([2]);
        let mut scheduler = TimingScheduler::new();

        scheduler.initialize(&mut dut)?;
        scheduler.time_slots = u64::MAX;

        assert_eq!(
            scheduler.advance_next(&mut dut),
            Err(TimingSchedulerError::StatisticsOverflow {
                time: SimulationTime::ZERO
            })
        );
        assert_eq!(dut.time, SimulationTime::ZERO);
        assert_eq!(dut.evaluations, vec![SimulationTime::ZERO]);

        Ok(())
    }
}

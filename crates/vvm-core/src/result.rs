use std::fmt;

use crate::{
    DetailedTestReport, IntoTestOutcome, ReplayToken, SimulationTime, TestOutcome, TestSummary,
};

/// Simulation operation stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStage {
    /// Driving any named clock inactive.
    DriveClockInactive,

    /// Driving the current stimulus.
    DriveStimulus,

    /// Evaluating while the primary clock is inactive.
    EvaluateInactive,

    /// Advancing simulation time while the primary clock is inactive.
    AdvanceInactivePhase,

    /// Driving any named clock active.
    DriveClockActive,

    /// Evaluating while the primary clock is active.
    EvaluateActive,

    /// Advancing simulation time while the primary clock is active.
    AdvanceActivePhase,

    /// Sampling DUT outputs.
    Sample,
}

impl fmt::Display for SimulationStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match *self {
            Self::DriveClockInactive => "driving a clock inactive",
            Self::DriveStimulus => "driving stimulus",
            Self::EvaluateInactive => "evaluating while the primary clock is inactive",
            Self::AdvanceInactivePhase => "advancing while the primary clock is inactive",
            Self::DriveClockActive => "driving a clock active",
            Self::EvaluateActive => "evaluating while the primary clock is active",
            Self::AdvanceActivePhase => "advancing while the primary clock is active",
            Self::Sample => "sampling DUT outputs",
        };

        f.write_str(description)
    }
}

/// One cycle-aware scoreboard failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckFailure<S, F> {
    /// Zero-based cycle number.
    cycle: u64,

    /// Simulation time.
    time: SimulationTime,

    /// Stimulus applied during the failing cycle.
    stimulus: S,

    /// Structured scoreboard error.
    error: F,
}

impl<S, F> CheckFailure<S, F> {
    /// Creates one check failure.
    pub(crate) const fn new(cycle: u64, time: SimulationTime, stimulus: S, error: F) -> Self {
        Self {
            cycle,
            time,
            stimulus,
            error,
        }
    }

    /// Returns the zero-based failing cycle.
    #[must_use]
    pub const fn cycle(&self) -> u64 {
        self.cycle
    }

    /// Returns the simulation time.
    #[must_use]
    pub const fn time(&self) -> SimulationTime {
        self.time
    }

    /// Returns the applied stimulus.
    #[must_use]
    pub const fn stimulus(&self) -> &S {
        &self.stimulus
    }

    /// Returns the scoreboard error.
    #[must_use]
    pub const fn error(&self) -> &F {
        &self.error
    }

    /// Consumes the failure.
    #[must_use]
    pub fn into_parts(self) -> (u64, SimulationTime, S, F) {
        (self.cycle, self.time, self.stimulus, self.error)
    }
}

impl<S, F> fmt::Display for CheckFailure<S, F>
where
    S: fmt::Debug,
    F: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cycle {} at {} with stimulus {:?}: {}",
            self.cycle, self.time, self.stimulus, self.error
        )
    }
}

/// Cycle-aware fatal simulation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationError<E> {
    /// Zero-based cycle during which the failure occurred.
    cycle: u64,

    /// Simulation time @ which the error occured.
    time: SimulationTime,

    /// Operation that failed.
    stage: SimulationStage,

    /// Named clock involved in the failure.
    clock_name: Option<String>,

    /// DUT error.
    source: E,
}

impl<E> SimulationError<E> {
    /// Creates a simulation failure.
    pub(crate) const fn new(
        cycle: u64,
        time: SimulationTime,
        stage: SimulationStage,
        source: E,
    ) -> Self {
        Self {
            cycle,
            time,
            stage,
            clock_name: None,
            source,
        }
    }

    /// Creates a clock-specific simulation failure.
    pub(crate) const fn new_for_clock(
        cycle: u64,
        time: SimulationTime,
        stage: SimulationStage,
        clock_name: String,
        source: E,
    ) -> Self {
        Self {
            cycle,
            time,
            stage,
            clock_name: Some(clock_name),
            source,
        }
    }

    /// Returns the zero-based failing cycle.
    #[must_use]
    pub const fn cycle(&self) -> u64 {
        self.cycle
    }

    /// Returns the simulation time @ which the error occurred.
    #[must_use]
    pub const fn time(&self) -> SimulationTime {
        self.time
    }

    /// Returns the failing simulation stage.
    #[must_use]
    pub const fn stage(&self) -> SimulationStage {
        self.stage
    }

    /// Returns the named clock involved in the failure.
    #[must_use]
    pub fn clock_name(&self) -> Option<&str> {
        self.clock_name.as_deref()
    }

    /// Returns the DUT error.
    #[must_use]
    pub const fn source_error(&self) -> &E {
        &self.source
    }

    /// Consumes the failure.
    #[must_use]
    pub fn into_parts(self) -> (u64, SimulationTime, SimulationStage, E) {
        (self.cycle, self.time, self.stage, self.source)
    }

    /// Consumes the failure and retains optional clock context.
    #[must_use]
    pub fn into_parts_with_clock(
        self,
    ) -> (u64, SimulationTime, SimulationStage, Option<String>, E) {
        (
            self.cycle,
            self.time,
            self.stage,
            self.clock_name,
            self.source,
        )
    }
}

// Clock-drive stages require optional clock-specific wording, unlike other stages.
impl<E> fmt::Display for SimulationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.clock_name.as_deref(), self.stage) {
            (Some(name), SimulationStage::DriveClockInactive) => write!(
                f,
                "simulation failed during primary cycle {} at {} while driving clock `{name}` \
                 inactive: {}",
                self.cycle, self.time, self.source
            ),
            (Some(name), SimulationStage::DriveClockActive) => write!(
                f,
                "simulation failed during primary cycle {} at {} while driving clock `{name}` \
                 active: {}",
                self.cycle, self.time, self.source
            ),
            _ => write!(
                f,
                "simulation failed during primary cycle {} at {} while {}: {}",
                self.cycle, self.time, self.stage, self.source
            ),
        }
    }
}

// The generic source must remain inspectable without imposing it on non-error displays.
impl<E> std::error::Error for SimulationError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Complete outcome of a synchronous testbench run.
#[derive(Debug)]
pub struct TestResult<S, F, E> {
    /// Number of primary transactions completed through scoreboard invocation.
    cycles: u64,

    /// Number of scoreboard checks performed.
    checks: u64,

    /// Retained check failures.
    failures: Vec<CheckFailure<S, F>>,

    /// Fatal simulation failure, if one occurred.
    simulation_error: Option<SimulationError<E>>,

    /// DUT finalization failure, if one occurred.
    finalization_error: Option<E>,

    /// Whether execution stopped because the failure limit was reached.
    stopped_by_failure_policy: bool,

    /// Simulation time when the testbench run began.
    start_time: SimulationTime,

    /// Simulation time when execution stopped.
    final_time: SimulationTime,

    /// Random-sequence replay metadata.
    replay_token: Option<ReplayToken>,
}

impl<S, F, E> TestResult<S, F, E> {
    /// Creates an empty test result.
    pub(crate) const fn new(start_time: SimulationTime, replay_token: Option<ReplayToken>) -> Self {
        Self {
            cycles: 0,
            checks: 0,
            failures: Vec::new(),
            simulation_error: None,
            finalization_error: None,
            stopped_by_failure_policy: false,
            start_time,
            final_time: start_time,
            replay_token,
        }
    }

    /// Returns the number of completed primary transactions.
    #[must_use]
    pub const fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Returns the number of scoreboard checks.
    #[must_use]
    pub const fn checks(&self) -> u64 {
        self.checks
    }

    /// Returns retained check failures.
    #[must_use]
    pub fn failures(&self) -> &[CheckFailure<S, F>] {
        &self.failures
    }

    /// Returns the number of retained check failures.
    #[must_use]
    pub const fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Returns the fatal simulation failure.
    #[must_use]
    pub const fn simulation_error(&self) -> Option<&SimulationError<E>> {
        self.simulation_error.as_ref()
    }

    /// Returns the finalization error.
    #[must_use]
    pub const fn finalization_error(&self) -> Option<&E> {
        self.finalization_error.as_ref()
    }

    /// Returns whether execution stopped at the configured failure limit.
    #[must_use]
    pub const fn stopped_by_failure_policy(&self) -> bool {
        self.stopped_by_failure_policy
    }

    /// Returns the simulation time at which execution began.
    #[must_use]
    pub const fn start_time(&self) -> SimulationTime {
        self.start_time
    }

    /// Returns the simulation time at which execution stopped.
    #[must_use]
    pub const fn final_time(&self) -> SimulationTime {
        self.final_time
    }

    /// Returns whether the test completed without any failure.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.failures.is_empty()
            && self.simulation_error.is_none()
            && self.finalization_error.is_none()
    }

    /// Returns the replay token for a randomized run.
    #[must_use]
    pub const fn replay_token(&self) -> Option<ReplayToken> {
        self.replay_token
    }

    /// Consumes the result and returns retained check failures.
    #[must_use]
    pub fn into_failures(self) -> Vec<CheckFailure<S, F>> {
        self.failures
    }

    /// Records one completed scoreboard check.
    pub(crate) const fn record_check(&mut self) {
        self.cycles = self.cycles.saturating_add(1);
        self.checks = self.checks.saturating_add(1);
    }

    /// Records one scoreboard failure.
    pub(crate) fn record_failure(&mut self, failure: CheckFailure<S, F>) {
        self.failures.push(failure);
    }

    /// Records a fatal simulation failure.
    pub(crate) fn record_simulation_error(&mut self, error: SimulationError<E>) {
        self.simulation_error = Some(error);
    }

    /// Records a finalization failure.
    pub(crate) fn record_finalization_error(&mut self, error: E) {
        self.finalization_error = Some(error);
    }

    /// Records policy-driven termination.
    pub(crate) const fn mark_stopped_by_failure_policy(&mut self) {
        self.stopped_by_failure_policy = true;
    }

    /// Records simulation time @ execution end.
    pub(crate) const fn record_final_time(&mut self, final_time: SimulationTime) {
        self.final_time = final_time;
    }

    /// Returns a compact, one-line result view.
    #[must_use]
    pub const fn summary(&self) -> TestSummary<'_, S, F, E> {
        TestSummary::new(self)
    }

    /// Returns a detailed multiline result view.
    #[must_use]
    pub const fn detailed_report(&self) -> DetailedTestReport<'_, S, F, E> {
        DetailedTestReport::new(self)
    }
}

impl<S, F, E> fmt::Display for TestResult<S, F, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.summary().fmt(f)
    }
}

impl<S, F, E, X> IntoTestOutcome for Result<TestResult<S, F, E>, X>
where
    S: fmt::Debug,
    F: fmt::Display,
    E: fmt::Display,
    X: fmt::Display,
{
    fn into_test_outcome(self) -> TestOutcome {
        match self {
            Ok(result) => TestOutcome::from_result(&result),

            Err(error) => TestOutcome::error(error),
        }
    }

    fn into_test_outcome_with_replay(self, replay: ReplayToken) -> TestOutcome {
        match self {
            Ok(result) => TestOutcome::from_result(&result),

            Err(error) => TestOutcome::error(error).with_replay_token(replay),
        }
    }
}

use std::fmt;

use crate::{
    DetailedTestReport, IntoTestOutcome, ReplayToken, SimulationTime, TestOutcome, TestSummary,
};

/// Simulation operation stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStage {
    /// Driving the clock inactive.
    DriveClockInactive,

    /// Driving the current stimulus.
    DriveStimulus,

    /// Evaluating the inactive clock phase.
    EvaluateInactive,

    /// Advancing through the inactive clock phase.
    AdvanceInactivePhase,

    /// Driving the clock active.
    DriveClockActive,

    /// Evaluating the active clock phase.
    EvaluateActive,

    /// Advancing through the active clock phase.
    AdvanceActivePhase,

    /// Sampling DUT outputs.
    Sample,
}

impl fmt::Display for SimulationStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match *self {
            Self::DriveClockInactive => "driving the clock inactive",
            Self::DriveStimulus => "driving stimulus",
            Self::EvaluateInactive => "evaluating the inactive clock phase",
            Self::AdvanceInactivePhase => "advancing through the inactive clock phase",
            Self::DriveClockActive => "driving the clock active",
            Self::EvaluateActive => "evaluating the active clock phase",
            Self::AdvanceActivePhase => "advancing through the active clock phase",
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
}

impl<E> fmt::Display for SimulationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "simulation failed during cycle {} at {} while {}: {}",
            self.cycle, self.time, self.stage, self.source
        )
    }
}

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
    /// Number of cycles completed through scoreboard invocation.
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

    /// Returns the number of completed cycles.
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

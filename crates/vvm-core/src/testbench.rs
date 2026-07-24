use crate::clock::scheduler::{ClockDriveFailure, ClockEventBatch, ClockPhase};
use crate::{
    CheckFailure, Clock, ClockScheduler, ClockTiming, CoverageModel, CycleTiming, Drive, Dut,
    FailurePolicy, ReferenceModel, ReplayToken, ReplayableSequence, Sample, Scoreboard,
    SimulationError, SimulationStage, SimulationTime, TestContext, TestResult,
};

/// Result type produced by a synchronous testbench run.
type RunResult<S, R, B, O, D> = TestResult<S, <B as Scoreboard<R, O>>::Error, <D as Dut>::Error>;

/// Read-only data from one successfully sampled testbench cycle.
#[derive(Debug, Clone, Copy)]
pub struct ObservedCycle<'a, S, O> {
    /// Zero-based transaction cycle.
    cycle: u64,
    /// Simulation time at which the observation was sampled.
    time: SimulationTime,
    /// Stimulus active during the sampled cycle.
    stimulus: &'a S,
    /// DUT observation sampled during the cycle.
    observed: &'a O,
}

impl<'a, S, O> ObservedCycle<'a, S, O> {
    /// Returns the zero-based transaction cycle.
    #[must_use]
    pub const fn cycle(self) -> u64 {
        self.cycle
    }

    /// Returns the simulation sampling time.
    #[must_use]
    pub const fn time(self) -> SimulationTime {
        self.time
    }

    /// Returns the active stimulus.
    #[must_use]
    pub const fn stimulus(self) -> &'a S {
        self.stimulus
    }

    /// Returns the sampled DUT observation.
    #[must_use]
    pub const fn observed(self) -> &'a O {
        self.observed
    }
}

/// Stage-tagged DUT failure from one runner operation.
struct StageError<E> {
    /// Failed operation.
    stage: SimulationStage,
    /// Named clock involved in the operation.
    clock_name: Option<String>,
    /// Underlying DUT error.
    source: E,
}

impl<E> StageError<E> {
    /// Creates a non-clock stage failure.
    const fn dut(stage: SimulationStage, source: E) -> Self {
        Self {
            stage,
            clock_name: None,
            source,
        }
    }

    /// Converts a scheduler clock-drive failure.
    fn clock(failure: ClockDriveFailure<E>) -> Self {
        let (clock_name, phase, source) = failure.into_parts();
        let stage = match phase {
            ClockPhase::Inactive => SimulationStage::DriveClockInactive,
            ClockPhase::Active => SimulationStage::DriveClockActive,
        };
        Self {
            stage,
            clock_name: Some(clock_name),
            source,
        }
    }

    /// Decomposes the failure.
    fn into_parts(self) -> (SimulationStage, Option<String>, E) {
        (self.stage, self.clock_name, self.source)
    }
}

/// Returns the time-advance failure stage for the current primary phase.
const fn advance_stage(phase: ClockPhase) -> SimulationStage {
    match phase {
        ClockPhase::Inactive => SimulationStage::AdvanceInactivePhase,
        ClockPhase::Active => SimulationStage::AdvanceActivePhase,
    }
}

/// Returns the evaluation failure stage for the current primary phase.
const fn evaluation_stage(phase: ClockPhase) -> SimulationStage {
    match phase {
        ClockPhase::Inactive => SimulationStage::EvaluateInactive,
        ClockPhase::Active => SimulationStage::EvaluateActive,
    }
}

/// Records one fatal simulation error in the current run.
fn record_stage_failure<S, F, E>(
    result: &mut TestResult<S, F, E>,
    cycle: u64,
    time: SimulationTime,
    error: StageError<E>,
) {
    let (stage, clock_name, source) = error.into_parts();
    let simulation_error = match clock_name {
        Some(clock_name) => SimulationError::new_for_clock(cycle, time, stage, clock_name, source),
        None => SimulationError::new(cycle, time, stage, source),
    };
    result.record_simulation_error(simulation_error);
}

/// Finalizes the DUT and records its final time even if finalization fails.
fn finalize_and_record<S, F, D>(dut: &mut D, result: &mut TestResult<S, F, D::Error>)
where
    D: Dut,
{
    if let Err(error) = Dut::finalize(dut) {
        result.record_finalization_error(error);
    }
    result.record_final_time(dut.simulation_time());
}

/// Advances to and drives every clock transition at the next event time.
fn advance_and_drive_next_batch<D>(
    dut: &mut D,
    clocks: &mut ClockScheduler<'_, D>,
) -> Result<ClockEventBatch, StageError<D::Error>>
where
    D: Dut,
{
    let phase = clocks.primary_phase();
    let elapsed = clocks.next_transition_after();
    Dut::advance_time(dut, elapsed)
        .map_err(|source| StageError::dut(advance_stage(phase), source))?;
    clocks.drive_next_batch(dut).map_err(StageError::clock)
}

/// Evaluates the DUT using the current primary semantic phase for diagnostics.
fn evaluate_current_phase<D>(
    dut: &mut D,
    clocks: &ClockScheduler<'_, D>,
) -> Result<(), StageError<D::Error>>
where
    D: Dut,
{
    Dut::evaluate(dut)
        .map_err(|source| StageError::dut(evaluation_stage(clocks.primary_phase()), source))
}

/// Initializes clocks and the first primary transaction at simulation time zero.
fn initialize_first_transaction<D, T>(
    dut: &mut D,
    clocks: &mut ClockScheduler<'_, D>,
    stimulus: &T,
) -> Result<(), StageError<D::Error>>
where
    D: Dut,
    T: Drive<D>,
{
    clocks
        .drive_initial_inactive(dut)
        .map_err(StageError::clock)?;
    stimulus
        .drive(dut)
        .map_err(|source| StageError::dut(SimulationStage::DriveStimulus, source))?;
    evaluate_current_phase(dut, clocks)
}

/// Processes events until the primary clock enters its active phase, then samples.
fn run_until_primary_active<D, O>(
    dut: &mut D,
    clocks: &mut ClockScheduler<'_, D>,
) -> Result<O, StageError<D::Error>>
where
    D: Dut,
    O: Sample<D>,
{
    loop {
        let batch = advance_and_drive_next_batch(dut, clocks)?;
        evaluate_current_phase(dut, clocks)?;
        if batch.primary_became_active() {
            return O::sample(dut)
                .map_err(|source| StageError::dut(SimulationStage::Sample, source));
        }
    }
}

/// Processes secondary events before advancing to the pending primary inactive boundary.
fn advance_to_primary_inactive_boundary<D>(
    dut: &mut D,
    clocks: &mut ClockScheduler<'_, D>,
) -> Result<(), StageError<D::Error>>
where
    D: Dut,
{
    loop {
        let next_event = clocks.next_transition_after();
        let primary_boundary = clocks.primary_transition_after();
        if next_event == primary_boundary {
            return Dut::advance_time(dut, primary_boundary)
                .map_err(|source| StageError::dut(SimulationStage::AdvanceActivePhase, source));
        }
        advance_and_drive_next_batch(dut, clocks)?;
        evaluate_current_phase(dut, clocks)?;
    }
}

/// Drives the pending primary inactive boundary, then starts the next transaction.
fn start_next_primary_cycle<D, T>(
    dut: &mut D,
    clocks: &mut ClockScheduler<'_, D>,
    stimulus: &T,
) -> Result<(), StageError<D::Error>>
where
    D: Dut,
    T: Drive<D>,
{
    let batch = clocks.drive_next_batch(dut).map_err(StageError::clock)?;
    if !batch.primary_became_inactive() {
        return evaluate_current_phase(dut, clocks);
    }
    stimulus
        .drive(dut)
        .map_err(|source| StageError::dut(SimulationStage::DriveStimulus, source))?;
    evaluate_current_phase(dut, clocks)
}

/// Marker for an unconfigured testbench component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Unconfigured;

/// Marker indicating no coverage model is configured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoCoverage;

/// Synchronous testbench with concrete component types.
pub struct Testbench<
    D,
    S = Unconfigured,
    R = Unconfigured,
    B = Unconfigured,
    C = Unconfigured,
    V = NoCoverage,
> {
    /// Testbench DUT.
    dut: D,
    /// Stimulus sequence.
    sequence: S,
    /// Reference model.
    reference_model: R,
    /// Scoreboard.
    scoreboard: B,
    /// Configured clock scheduler.
    clocks: C,
    /// Typed functional coverage model.
    coverage: V,
    /// Check-failure retention and stopping policy.
    failure_policy: FailurePolicy,
    /// Optional compatibility override for primary timing.
    primary_cycle_timing: Option<CycleTiming>,
    /// Replay metadata for the configured sequence.
    replay_token: Option<ReplayToken>,
}

impl<D> Testbench<D> {
    /// Creates a testbench around the supplied DUT.
    #[must_use]
    pub const fn new(dut: D) -> Self {
        Self {
            dut,
            sequence: Unconfigured,
            reference_model: Unconfigured,
            scoreboard: Unconfigured,
            clocks: Unconfigured,
            coverage: NoCoverage,
            failure_policy: FailurePolicy::STOP_ON_FIRST,
            primary_cycle_timing: None,
            replay_token: None,
        }
    }
}

impl<D, S, R, B, C, V> Testbench<D, S, R, B, C, V> {
    /// Configures the stimulus sequence.
    #[must_use]
    pub fn with_sequence<NS>(self, sequence: NS) -> Testbench<D, NS, R, B, C, V> {
        Testbench {
            dut: self.dut,
            sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clocks: self.clocks,
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: None,
        }
    }

    /// Configures a random or otherwise replayable sequence.
    #[must_use]
    pub fn with_replayable_sequence<NS>(self, sequence: NS) -> Testbench<D, NS, R, B, C, V>
    where
        NS: ReplayableSequence,
    {
        let replay_token = Some(sequence.replay_token());
        Testbench {
            dut: self.dut,
            sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clocks: self.clocks,
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token,
        }
    }

    /// Configures the reference model.
    #[must_use]
    pub fn with_reference_model<NR>(self, reference_model: NR) -> Testbench<D, S, NR, B, C, V> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model,
            scoreboard: self.scoreboard,
            clocks: self.clocks,
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: self.replay_token,
        }
    }

    /// Configures the scoreboard.
    #[must_use]
    pub fn with_scoreboard<NB>(self, scoreboard: NB) -> Testbench<D, S, R, NB, C, V> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard,
            clocks: self.clocks,
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: self.replay_token,
        }
    }

    /// Configures one primary clock through the compatibility scheduler path.
    #[must_use]
    pub fn with_clock<'clock, NC>(
        self,
        clock: NC,
    ) -> Testbench<D, S, R, B, ClockScheduler<'clock, D>, V>
    where
        D: Dut,
        NC: Clock<D> + 'clock,
    {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clocks: ClockScheduler::single(clock, ClockTiming::UNIT),
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: self.replay_token,
        }
    }

    /// Configures independently timed primary and secondary clocks.
    #[must_use]
    pub fn with_clocks(
        self,
        clocks: ClockScheduler<'_, D>,
    ) -> Testbench<D, S, R, B, ClockScheduler<'_, D>, V>
    where
        D: Dut,
    {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clocks,
            coverage: self.coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: self.replay_token,
        }
    }

    /// Configures check-failure collection and stopping behavior.
    #[must_use]
    pub const fn with_failure_policy(mut self, failure_policy: FailurePolicy) -> Self {
        self.failure_policy = failure_policy;
        self
    }

    /// Overrides recurring timing of the primary clock.
    #[must_use]
    pub const fn with_primary_cycle_timing(mut self, cycle_timing: CycleTiming) -> Self {
        self.primary_cycle_timing = Some(cycle_timing);
        self
    }

    /// Overrides recurring timing of the primary clock.
    #[must_use]
    pub const fn with_cycle_timing(self, cycle_timing: CycleTiming) -> Self {
        self.with_primary_cycle_timing(cycle_timing)
    }
}

impl<D, S, R, B, C> Testbench<D, S, R, B, C, NoCoverage> {
    /// Configures one typed functional-coverage model.
    ///
    /// A testbench owns one coverage model. Use [`Self::run_covered`] to sample
    /// it at the normal observer point and capture its final or partial state.
    #[must_use]
    pub fn with_coverage<V>(self, coverage: V) -> Testbench<D, S, R, B, C, V> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clocks: self.clocks,
            coverage,
            failure_policy: self.failure_policy,
            primary_cycle_timing: self.primary_cycle_timing,
            replay_token: self.replay_token,
        }
    }
}

impl<D, S, R, B, V> Testbench<D, S, R, B, ClockScheduler<'_, D>, V>
where
    D: Dut,
    S: IntoIterator,
    S::Item: Drive<D>,
    R: ReferenceModel<S::Item>,
{
    /// Runs scheduler-driven primary-clock transactions.
    #[must_use]
    pub fn run<O>(self) -> RunResult<S::Item, R::Expected, B, O, D>
    where
        O: Sample<D>,
        B: Scoreboard<R::Expected, O>,
    {
        self.execute_with_observer::<O, _>(|_| {})
    }

    /// Runs the testbench while synchronously observing every successfully
    /// sampled transaction.
    ///
    /// The observer is called after DUT observation sampling and before
    /// reference-model prediction and scoreboard checking. It receives only
    /// immutable transaction data and cannot alter runner scheduling.
    #[must_use]
    pub fn run_with_observer<O, F>(
        self,
        cycle_observer: F,
    ) -> RunResult<S::Item, R::Expected, B, O, D>
    where
        O: Sample<D>,
        B: Scoreboard<R::Expected, O>,
        F: for<'sample> FnMut(ObservedCycle<'sample, S::Item, O>),
    {
        self.execute_with_observer::<O, F>(cycle_observer)
    }

    /// Runs the shared simulation kernel with an observation callback.
    fn execute_with_observer<O, F>(
        self,
        mut cycle_observer: F,
    ) -> RunResult<S::Item, R::Expected, B, O, D>
    where
        O: Sample<D>,
        B: Scoreboard<R::Expected, O>,
        F: for<'sample> FnMut(ObservedCycle<'sample, S::Item, O>),
    {
        let Self {
            mut dut,
            sequence,
            mut reference_model,
            mut scoreboard,
            mut clocks,
            failure_policy,
            primary_cycle_timing,
            replay_token,
            ..
        } = self;

        if let Some(cycle_timing) = primary_cycle_timing {
            clocks.replace_primary_timing(ClockTiming::from_cycle(cycle_timing));
        }

        let mut result = TestResult::new(dut.simulation_time(), replay_token);
        let mut sequence = sequence.into_iter();

        let Some(first_stimulus) = sequence.next() else {
            finalize_and_record(&mut dut, &mut result);
            return result;
        };

        let mut stimulus = Some(first_stimulus);

        let Some(current_stimulus) = stimulus.as_ref() else {
            finalize_and_record(&mut dut, &mut result);
            return result;
        };

        if let Err(error) = initialize_first_transaction(&mut dut, &mut clocks, current_stimulus) {
            record_stage_failure(&mut result, 0, dut.simulation_time(), error);
            finalize_and_record(&mut dut, &mut result);
            return result;
        }

        loop {
            let cycle = result.cycles();

            let observed = match run_until_primary_active::<D, O>(&mut dut, &mut clocks) {
                Ok(observed) => observed,
                Err(error) => {
                    record_stage_failure(&mut result, cycle, dut.simulation_time(), error);
                    break;
                }
            };

            let Some(active_stimulus) = stimulus.as_ref() else {
                break;
            };

            let check_time = dut.simulation_time();
            cycle_observer(ObservedCycle {
                cycle,
                time: check_time,
                stimulus: active_stimulus,
                observed: &observed,
            });

            let expected = reference_model.predict(active_stimulus);
            let check_result = scoreboard.check(expected, observed);

            result.record_check();

            if let Err(error) = check_result {
                let Some(failed_stimulus) = stimulus.take() else {
                    break;
                };

                result.record_failure(CheckFailure::new(cycle, check_time, failed_stimulus, error));

                if failure_policy.should_stop(result.failure_count()) {
                    result.mark_stopped_by_failure_policy();
                    break;
                }
            }

            if let Err(error) = advance_to_primary_inactive_boundary(&mut dut, &mut clocks) {
                record_stage_failure(&mut result, cycle, dut.simulation_time(), error);
                break;
            }

            let Some(next_stimulus) = sequence.next() else {
                break;
            };

            let next_cycle = result.cycles();

            if let Err(error) = start_next_primary_cycle(&mut dut, &mut clocks, &next_stimulus) {
                record_stage_failure(&mut result, next_cycle, dut.simulation_time(), error);
                break;
            }

            stimulus = Some(next_stimulus);
        }

        finalize_and_record(&mut dut, &mut result);
        result
    }
    /// Runs the testbench, samples coverage, and captures the resulting group.
    ///
    /// Sampling occurs after successful DUT observation and before scoreboard
    /// checking, matching [`Self::run_with_observer`]. The first coverage error
    /// is deferred through `context`; simulation continues and partial coverage
    /// is captured. This does not merge or report coverage across tests.
    #[must_use]
    pub fn run_covered<O>(
        self,
        context: &mut TestContext,
    ) -> RunResult<S::Item, R::Expected, B, O, D>
    where
        O: Sample<D>,
        B: Scoreboard<R::Expected, O>,
        V: CoverageModel<S::Item, O>,
    {
        let Self {
            dut,
            sequence,
            reference_model,
            scoreboard,
            clocks,
            mut coverage,
            failure_policy,
            primary_cycle_timing,
            replay_token,
        } = self;
        let uncovered = Testbench {
            dut,
            sequence,
            reference_model,
            scoreboard,
            clocks,
            coverage: NoCoverage,
            failure_policy,
            primary_cycle_timing,
            replay_token,
        };
        let mut sampling_error = None;

        let result = uncovered.execute_with_observer::<O, _>(|cycle| {
            if sampling_error.is_some() {
                return;
            }

            if let Err(error) = coverage.sample(cycle) {
                sampling_error = Some(error);
            }
        });

        if let Some(error) = sampling_error.as_ref() {
            context.record_coverage_sampling_error(error);
        }

        if let Err(error) = context.capture_coverage(&coverage) {
            context.record_coverage_capture_error(&error);
        }

        result
    }
}

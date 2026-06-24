use crate::{
    CheckFailure, Clock, CycleTiming, Drive, Dut, FailurePolicy, ReferenceModel, Sample,
    Scoreboard, SimulationError, SimulationStage, SimulationTime, TestResult, TimeStep,
};

/// Result type produced by a synchronous testbench run.
type RunResult<S, R, B, O, D> = TestResult<S, <B as Scoreboard<R, O>>::Error, <D as Dut>::Error>;

/// Stage-tagged DUT error from one runner step.
type StageError<E> = (SimulationStage, E);

/// Records one fatal simulation error in the current run.
fn record_simulation_failure<S, F, E>(
    result: &mut TestResult<S, F, E>,
    cycle: u64,
    time: SimulationTime,
    stage: SimulationStage,
    source: E,
) {
    result.record_simulation_error(SimulationError::new(cycle, time, stage, source));
}

/// Marker for an unconfigured testbench component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Unconfigured;

/// Synchronous testbench with concrete component types.
pub struct Testbench<D, S = Unconfigured, R = Unconfigured, B = Unconfigured, C = Unconfigured> {
    /// Testbench DUT.
    dut: D,

    /// Stimulus sequence.
    sequence: S,

    /// Reference model.
    reference_model: R,

    /// Scoreboard.
    scoreboard: B,

    /// Clock driver.
    clock: C,

    /// Check-failure retention and stopping policy.
    failure_policy: FailurePolicy,

    /// Testbench clock cycle timing.
    cycle_timing: CycleTiming,
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
            clock: Unconfigured,
            failure_policy: FailurePolicy::STOP_ON_FIRST,
            cycle_timing: CycleTiming::default(),
        }
    }
}

impl<D, S, R, B, C> Testbench<D, S, R, B, C> {
    /// Configures the stimulus sequence.
    #[must_use]
    pub fn with_sequence<NS>(self, sequence: NS) -> Testbench<D, NS, R, B, C> {
        Testbench {
            dut: self.dut,
            sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clock: self.clock,
            failure_policy: self.failure_policy,
            cycle_timing: self.cycle_timing,
        }
    }

    /// Configures the reference model.
    #[must_use]
    pub fn with_reference_model<NR>(self, reference_model: NR) -> Testbench<D, S, NR, B, C> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model,
            scoreboard: self.scoreboard,
            clock: self.clock,
            failure_policy: self.failure_policy,
            cycle_timing: self.cycle_timing,
        }
    }

    /// Configures the scoreboard.
    #[must_use]
    pub fn with_scoreboard<NB>(self, scoreboard: NB) -> Testbench<D, S, R, NB, C> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard,
            clock: self.clock,
            failure_policy: self.failure_policy,
            cycle_timing: self.cycle_timing,
        }
    }

    /// Configures the clock driver.
    #[must_use]
    pub fn with_clock<NC>(self, clock: NC) -> Testbench<D, S, R, B, NC> {
        Testbench {
            dut: self.dut,
            sequence: self.sequence,
            reference_model: self.reference_model,
            scoreboard: self.scoreboard,
            clock,
            failure_policy: self.failure_policy,
            cycle_timing: self.cycle_timing,
        }
    }

    /// Configures check-failure collection and stopping behavior.
    #[must_use]
    pub const fn with_failure_policy(mut self, failure_policy: FailurePolicy) -> Self {
        self.failure_policy = failure_policy;
        self
    }

    /// Configures inactive and active clock-phase durations.
    #[must_use]
    pub const fn with_cycle_timing(mut self, cycle_timing: CycleTiming) -> Self {
        self.cycle_timing = cycle_timing;
        self
    }
}

impl<D, S, R, B, C> Testbench<D, S, R, B, C>
where
    D: Dut,
    S: IntoIterator,
    S::Item: Drive<D>,
    R: ReferenceModel<S::Item>,
    C: Clock<D>,
{
    /// Runs the configured synchronous testbench.
    ///
    /// The cycle order is:
    ///
    /// 1. drive the clock inactive;
    /// 2. drive stimulus;
    /// 3. evaluate the inactive phase;
    /// 4. drive the clock active;
    /// 5. evaluate the active phase;
    /// 6. sample outputs;
    /// 7. predict expected outputs;
    /// 8. invoke the scoreboard.
    ///
    /// DUT access errors stop simulation immediately. Scoreboard failures obey
    /// the configured [`FailurePolicy`]. The DUT is always offered one explicit
    /// finalization call before this method returns.
    fn run_inactive_phase(
        dut: &mut D,
        clock: &mut C,
        stimulus: &S::Item,
        inactive_phase: TimeStep,
    ) -> Result<(), StageError<D::Error>> {
        clock
            .drive_inactive(dut)
            .map_err(|source| (SimulationStage::DriveClockInactive, source))?;
        stimulus
            .drive(dut)
            .map_err(|source| (SimulationStage::DriveStimulus, source))?;
        Dut::evaluate(dut).map_err(|source| (SimulationStage::EvaluateInactive, source))?;
        Dut::advance_time(dut, inactive_phase)
            .map_err(|source| (SimulationStage::AdvanceInactivePhase, source))?;

        Ok(())
    }

    /// Drives, evaluates, and samples the active edge of one cycle.
    fn run_active_phase<O>(dut: &mut D, clock: &mut C) -> Result<O, StageError<D::Error>>
    where
        O: Sample<D>,
    {
        clock
            .drive_active(dut)
            .map_err(|source| (SimulationStage::DriveClockActive, source))?;
        Dut::evaluate(dut).map_err(|source| (SimulationStage::EvaluateActive, source))?;

        O::sample(dut).map_err(|source| (SimulationStage::Sample, source))
    }

    /// Runs the configured synchronous testbench.
    #[must_use]
    pub fn run<O>(self) -> RunResult<S::Item, R::Expected, B, O, D>
    where
        O: Sample<D>,
        B: Scoreboard<R::Expected, O>,
    {
        let Self {
            mut dut,
            sequence,
            mut reference_model,
            mut scoreboard,
            mut clock,
            failure_policy,
            cycle_timing,
        } = self;

        let mut result = TestResult::new(dut.simulation_time());

        for stimulus in sequence {
            let cycle = result.cycles();

            if let Err((stage, source)) = Self::run_inactive_phase(
                &mut dut,
                &mut clock,
                &stimulus,
                cycle_timing.inactive_phase(),
            ) {
                record_simulation_failure(&mut result, cycle, dut.simulation_time(), stage, source);
                break;
            }

            let observed = match Self::run_active_phase::<O>(&mut dut, &mut clock) {
                Ok(observed) => observed,
                Err((stage, source)) => {
                    record_simulation_failure(
                        &mut result,
                        cycle,
                        dut.simulation_time(),
                        stage,
                        source,
                    );
                    break;
                }
            };

            // 8. Predict and check at the same time.
            let expected = reference_model.predict(&stimulus);

            let check_time = dut.simulation_time();

            let check_result = scoreboard.check(expected, observed);

            result.record_check();

            if let Err(error) = check_result {
                result.record_failure(CheckFailure::new(cycle, check_time, stimulus, error));

                if failure_policy.should_stop(result.failure_count()) {
                    result.mark_stopped_by_failure_policy();
                    break;
                }
            }

            // 9. Complete the active phase before the next cycle.
            if let Err(source) = Dut::advance_time(&mut dut, cycle_timing.active_phase()) {
                record_simulation_failure(
                    &mut result,
                    cycle,
                    dut.simulation_time(),
                    SimulationStage::AdvanceActivePhase,
                    source,
                );
                break;
            }
        }

        if let Err(error) = Dut::finalize(&mut dut) {
            result.record_finalization_error(error);
        }

        result.record_final_time(dut.simulation_time());

        result
    }
}

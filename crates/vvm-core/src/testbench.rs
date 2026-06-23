use crate::{
    CheckFailure, Clock, Drive, Dut, FailurePolicy, ReferenceModel, Sample, Scoreboard,
    SimulationError, SimulationStage, TestResult,
};

/// Result type produced by a synchronous testbench run.
type RunResult<S, R, B, O, D> = TestResult<S, <B as Scoreboard<R, O>>::Error, <D as Dut>::Error>;

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
        }
    }

    /// Configures check-failure collection and stopping behavior.
    #[must_use]
    pub const fn with_failure_policy(mut self, failure_policy: FailurePolicy) -> Self {
        self.failure_policy = failure_policy;
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
        } = self;

        let mut result = TestResult::new();

        for stimulus in sequence {
            let cycle = result.cycles();

            if let Err(source) = clock.drive_inactive(&mut dut) {
                result.record_simulation_error(SimulationError::new(
                    cycle,
                    SimulationStage::DriveClockInactive,
                    source,
                ));
                break;
            }

            if let Err(source) = stimulus.drive(&mut dut) {
                result.record_simulation_error(SimulationError::new(
                    cycle,
                    SimulationStage::DriveStimulus,
                    source,
                ));
                break;
            }

            if let Err(source) = Dut::evaluate(&mut dut) {
                result.record_simulation_error(SimulationError::new(
                    cycle,
                    SimulationStage::EvaluateInactive,
                    source,
                ));
                break;
            }

            if let Err(source) = clock.drive_active(&mut dut) {
                result.record_simulation_error(SimulationError::new(
                    cycle,
                    SimulationStage::DriveClockActive,
                    source,
                ));
                break;
            }

            if let Err(source) = Dut::evaluate(&mut dut) {
                result.record_simulation_error(SimulationError::new(
                    cycle,
                    SimulationStage::EvaluateActive,
                    source,
                ));
                break;
            }

            let observed = match O::sample(&dut) {
                Ok(observed) => observed,
                Err(source) => {
                    result.record_simulation_error(SimulationError::new(
                        cycle,
                        SimulationStage::Sample,
                        source,
                    ));
                    break;
                }
            };

            let expected = reference_model.predict(&stimulus);

            let check_result = scoreboard.check(expected, observed);

            result.record_check();

            if let Err(error) = check_result {
                result.record_failure(CheckFailure::new(cycle, stimulus, error));

                if failure_policy.should_stop(result.failure_count()) {
                    result.mark_stopped_by_failure_policy();
                    break;
                }
            }
        }

        if let Err(error) = Dut::finalize(&mut dut) {
            result.record_finalization_error(error);
        }

        result
    }
}

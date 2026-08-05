//! A one-clock FIFO verified with typed transactions and a queue model.

// `build.rs` writes this safe generated wrapper below OUT_DIR.
vvm::include_dut!(sync_fifo);

#[doc(hidden)]
pub mod benchmark;

/// FIFO verification implementation and scenario tests.
#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use thiserror::Error;
    use vvm::coverage::{Bin, BuildError, Coverpoint, Cross2, DefinitionError};
    use vvm::dut::Sample;
    use vvm::random::{RandomContext, ReplayToken, ReplayableSequence, Seed};
    use vvm::test::{TestContext, TestRunConfig};
    use vvm::testbench::{
        ExactScoreboard, Mismatch, ObservedCycle, ReferenceModel, TestResult, Testbench,
    };
    use vvm::{Clock, Drive};

    use crate::sync_fifo::{SyncFifo, SyncFifoError};

    /// Number of entries modeled by both the RTL and Rust queue.
    const DEPTH: usize = 8;
    /// Default length of the randomized regression.
    const RANDOM_CYCLES: u64 = 1_000;
    /// Stable random stream used unless a user supplies `VVM_REPLAY` or `VVM_SEED`.
    const DEFAULT_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x46_49_46_4f));

    /// Errors surfaced by the synchronous FIFO verification harness.
    #[derive(Debug, Error)]
    enum Error {
        #[error(transparent)]
        Dut(#[from] SyncFifoError),
        #[error(transparent)]
        Coverage(#[from] BuildError),
        #[error(transparent)]
        CoverageDefinition(#[from] DefinitionError),
    }

    /// Result type used by FIFO tests and setup helpers.
    type Result<T> = std::result::Result<T, Error>;
    /// Runner result with FIFO transactions, observations, and DUT errors.
    type FifoResult =
        TestResult<FifoTransaction, Mismatch<FifoObservation, FifoObservation>, SyncFifoError>;

    /// Typed rising-edge driver for the FIFO clock port.
    #[derive(Clone, Copy, Debug, Default, Clock)]
    #[vvm(dut = SyncFifo, clock = "clk")]
    struct FifoClock;

    /// Inputs presented for one FIFO clock edge.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Drive)]
    #[vvm(dut = SyncFifo)]
    struct FifoTransaction {
        #[vvm(port)]
        reset_n: bool,
        #[vvm(port)]
        push: bool,
        #[vvm(port)]
        push_data: u8,
        #[vvm(port)]
        pop: bool,
    }

    impl FifoTransaction {
        /// Returns the mandatory active-low reset transaction.
        const fn reset() -> Self {
            Self {
                reset_n: false,
                push: false,
                push_data: 0,
                pop: false,
            }
        }

        /// Returns one normal push/pop request with reset released.
        const fn operation(push: bool, push_data: u8, pop: bool) -> Self {
            Self {
                reset_n: true,
                push,
                push_data,
                pop,
            }
        }

        /// Maps independent request bits into one coverage operation category.
        const fn operation_kind(self) -> FifoOperation {
            match (self.push, self.pop) {
                (false, false) => FifoOperation::Idle,
                (true, false) => FifoOperation::Push,
                (false, true) => FifoOperation::Pop,
                (true, true) => FifoOperation::Simultaneous,
            }
        }
    }

    /// Outputs sampled after a FIFO clock edge.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FifoObservation {
        pop_data: u8,
        occupancy: u8,
        boundary: Boundary,
        acceptance: Acceptance,
    }

    /// Occupancy category derived from the sampled status flags.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Boundary {
        Empty,
        Available,
        Full,
    }

    /// Acceptance result reported for the request at the sampled edge.
    /// Semantic operation extracted from push and pop request bits.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Acceptance {
        Accepted,
        Overflow,
        Underflow,
    }

    /// Semantic operation extracted from push and pop request bits.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FifoOperation {
        Idle,
        Push,
        Pop,
        Simultaneous,
    }

    impl Sample<SyncFifo> for FifoObservation {
        /// Reads generated outputs without evaluating or advancing the DUT.
        fn sample(dut: &SyncFifo) -> std::result::Result<Self, SyncFifoError> {
            // Keep the public observation semantic: flags become a boundary
            // state and one accepted/rejected result instead of four booleans.
            let occupancy = dut.occupancy()?;
            let boundary = if dut.empty()? {
                Boundary::Empty
            } else if dut.full()? {
                Boundary::Full
            } else {
                Boundary::Available
            };
            let acceptance = if dut.overflow()? {
                Acceptance::Overflow
            } else if dut.underflow()? {
                Acceptance::Underflow
            } else {
                Acceptance::Accepted
            };

            Ok(Self {
                pop_data: dut.pop_data()?,
                occupancy,
                boundary,
                acceptance,
            })
        }
    }

    /// Functional coverage for operation types and observed FIFO boundaries.
    #[derive(vvm::Coverage)]
    #[vvm(definition = "sync_fifo_coverage", revision = 1, stimulus = FifoTransaction, observation = FifoObservation)]
    struct FifoCoverage {
        #[vvm(coverpoint(build = operation_coverpoint, sample = operation_for))]
        operation: Coverpoint<FifoOperation>,
        #[vvm(coverpoint(build = boundary_coverpoint, sample = boundary_for))]
        boundary: Coverpoint<Boundary>,
        #[vvm(cross(left = operation, right = boundary))]
        operation_x_boundary: Cross2,
    }

    /// Builds bins for every externally visible request combination.
    fn operation_coverpoint(
        name: &'static str,
    ) -> std::result::Result<Coverpoint<FifoOperation>, BuildError> {
        Coverpoint::builder(name)
            .bin(Bin::value("idle", FifoOperation::Idle))
            .bin(Bin::value("push", FifoOperation::Push))
            .bin(Bin::value("pop", FifoOperation::Pop))
            .bin(Bin::value("simultaneous", FifoOperation::Simultaneous))
            .build()
    }

    /// Builds bins for the three FIFO occupancy boundaries.
    fn boundary_coverpoint(
        name: &'static str,
    ) -> std::result::Result<Coverpoint<Boundary>, BuildError> {
        Coverpoint::builder(name)
            .bin(Bin::value("empty", Boundary::Empty))
            .bin(Bin::value("available", Boundary::Available))
            .bin(Bin::value("full", Boundary::Full))
            .build()
    }

    /// Extracts the transaction operation for coverage sampling.
    const fn operation_for(
        cycle: ObservedCycle<'_, FifoTransaction, FifoObservation>,
    ) -> FifoOperation {
        cycle.stimulus().operation_kind()
    }

    /// Extracts the sampled boundary state for coverage sampling.
    const fn boundary_for(cycle: ObservedCycle<'_, FifoTransaction, FifoObservation>) -> Boundary {
        cycle.observed().boundary
    }

    /// Logical FIFO behavior, deliberately independent of its pointer implementation.
    #[derive(Default)]
    struct FifoModel {
        queue: VecDeque<u8>,
        pop_data: u8,
    }

    impl ReferenceModel<FifoTransaction> for FifoModel {
        type Expected = FifoObservation;

        /// Applies one pre-edge FIFO contract decision to the logical queue.
        fn predict(&mut self, transaction: &FifoTransaction) -> Self::Expected {
            // The model describes the queue contract, not the DUT's pointers.
            if !transaction.reset_n {
                self.queue.clear();
                self.pop_data = 0;
                return self.observation(false, false);
            }

            let was_empty = self.queue.is_empty();
            let was_full = self.queue.len() == DEPTH;
            // Decisions use pre-edge state, matching the registered HDL flags.
            let push_accepted = transaction.push && !was_full;
            let pop_accepted = transaction.pop && !was_empty;

            if pop_accepted {
                self.pop_data = self.queue.pop_front().unwrap_or_default();
            }

            if push_accepted {
                self.queue.push_back(transaction.push_data);
            }

            self.observation(transaction.push && was_full, transaction.pop && was_empty)
        }
    }

    impl FifoModel {
        /// Builds the expected observation after queue mutation.
        fn observation(&self, overflow: bool, underflow: bool) -> FifoObservation {
            FifoObservation {
                pop_data: self.pop_data,
                occupancy: u8::try_from(self.queue.len()).unwrap_or_default(),
                boundary: if self.queue.is_empty() {
                    Boundary::Empty
                } else if self.queue.len() == DEPTH {
                    Boundary::Full
                } else {
                    Boundary::Available
                },
                acceptance: if overflow {
                    Acceptance::Overflow
                } else if underflow {
                    Acceptance::Underflow
                } else {
                    Acceptance::Accepted
                },
            }
        }
    }

    /// Replayable stream that generates one FIFO request per clock edge.
    struct RandomSequence {
        random: RandomContext,
        remaining: u64,
        reset_pending: bool,
    }

    impl RandomSequence {
        /// Creates a replayable sequence with one reset transaction first.
        fn new(replay: ReplayToken, cycles: u64) -> Self {
            Self {
                random: RandomContext::from_replay(replay),
                remaining: cycles,
                reset_pending: cycles != 0,
            }
        }
    }

    impl ReplayableSequence for RandomSequence {
        /// Returns the stream token needed to reproduce subsequent requests.
        fn replay_token(&self) -> ReplayToken {
            self.random.replay_token()
        }
    }

    impl Iterator for RandomSequence {
        type Item = FifoTransaction;

        /// Produces one deterministic FIFO request from the replay stream.
        fn next(&mut self) -> Option<Self::Item> {
            self.remaining = self.remaining.checked_sub(1)?;

            if self.reset_pending {
                // Every replay stream begins from a known FIFO state.
                self.reset_pending = false;
                return Some(FifoTransaction::reset());
            }

            let bits = self.random.next_u32();
            Some(FifoTransaction::operation(
                bits & 1 != 0,
                u8::try_from((bits >> 8) & 0xff).unwrap_or_default(),
                bits & 2 != 0,
            ))
        }
    }

    /// Runs an uncovered FIFO sequence, optionally configuring a VCD first.
    fn run(
        sequence: impl Iterator<Item = FifoTransaction>,
        config: Option<&TestRunConfig>,
    ) -> Result<FifoResult> {
        let mut dut = SyncFifo::new()?;

        if let Some(config) = config {
            config.configure_trace(&mut dut)?;
        }

        Ok(Testbench::new(dut)
            .with_sequence(sequence)
            .with_reference_model(FifoModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(FifoClock)
            .run::<FifoObservation>())
    }

    /// Runs a FIFO sequence while persisting the attached coverage model.
    fn run_covered(
        sequence: impl Iterator<Item = FifoTransaction>,
        context: &mut TestContext,
        trace: bool,
    ) -> Result<FifoResult> {
        let mut dut = SyncFifo::new()?;

        if trace {
            // Tracing is configured before the runner performs its first eval.
            context.config().configure_trace(&mut dut)?;
        }

        Ok(Testbench::new(dut)
            .with_sequence(sequence)
            .with_reference_model(FifoModel::default())
            .with_scoreboard(ExactScoreboard)
            .with_clock(FifoClock)
            .with_coverage(FifoCoverage::new("dut.sync_fifo")?)
            .run_covered::<FifoObservation>(context))
    }

    /// Verifies reset, ordered data transfer, and one simultaneous operation.
    #[vvm::test(trace, coverage)]
    fn fifo_smoke(context: &mut TestContext) -> Result<FifoResult> {
        run_covered(
            [
                FifoTransaction::reset(),
                FifoTransaction::operation(true, 0x12, false),
                FifoTransaction::operation(true, 0x34, true),
                FifoTransaction::operation(false, 0, true),
            ]
            .into_iter(),
            context,
            true,
        )
    }

    #[test]
    fn fifo_boundary_conditions() -> Result<()> {
        let mut sequence = vec![
            FifoTransaction::reset(),
            FifoTransaction::operation(false, 0, true),
        ];
        sequence.extend((0..DEPTH).map(|value| {
            FifoTransaction::operation(true, u8::try_from(value).unwrap_or_default(), false)
        }));
        sequence.push(FifoTransaction::operation(true, 0xff, false));
        sequence.extend((0..DEPTH).map(|_| FifoTransaction::operation(false, 0, true)));

        assert!(run(sequence.into_iter(), None)?.passed());
        Ok(())
    }

    #[test]
    fn fifo_simultaneous_operations() -> Result<()> {
        let sequence = [
            FifoTransaction::reset(),
            FifoTransaction::operation(true, 1, false),
            FifoTransaction::operation(true, 2, true),
            FifoTransaction::operation(false, 0, true),
        ];
        assert!(run(sequence.into_iter(), None)?.passed());
        Ok(())
    }

    #[test]
    fn fifo_wraparound() -> Result<()> {
        let mut sequence = vec![FifoTransaction::reset()];
        for value in 0..24_u8 {
            sequence.push(FifoTransaction::operation(true, value, false));
            sequence.push(FifoTransaction::operation(false, 0, true));
        }
        assert!(run(sequence.into_iter(), None)?.passed());
        Ok(())
    }

    /// Runs deterministic randomized FIFO traffic and records its replay token.
    #[vvm::test(cycles, replay(default = DEFAULT_REPLAY), coverage)]
    fn fifo_random(context: &mut TestContext) -> Result<FifoResult> {
        let config = context.config();
        run_covered(
            RandomSequence::new(
                config.replay_token_or(DEFAULT_REPLAY),
                config.cycles_or(RANDOM_CYCLES),
            ),
            context,
            false,
        )
    }
}

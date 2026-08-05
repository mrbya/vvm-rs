//! Hidden native benchmark harness for the synchronous FIFO example.

use std::collections::VecDeque;

use vvm::coverage::{Bin, Coverpoint, Cross2};
use vvm::dut::Sample;
use vvm::random::{RandomContext, ReplayToken, ReplayableSequence, Seed};
use vvm::testbench::{ExactScoreboard, Mismatch, ObservedCycle, ReferenceModel, TestResult, Testbench};
use vvm::{Clock, Drive};

use crate::sync_fifo::{SyncFifo, SyncFifoError};

/// Benchmark replay token.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0x46_49_46_4f));
/// Number of deterministic cycles used by the benchmark workload.
pub const BENCH_CYCLES: u64 = 512;
/// FIFO depth mirrored by the logical model.
const DEPTH: usize = 8;

/// Benchmark result type.
pub type FifoResult = TestResult<FifoTransaction, Mismatch<FifoObservation, FifoObservation>, SyncFifoError>;

/// Typed rising-edge driver for the FIFO clock port.
#[derive(Clone, Copy, Debug, Default, Clock)]
#[vvm(dut = SyncFifo, clock = "clk")]
struct FifoClock;

/// Inputs presented for one FIFO clock edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Drive)]
#[vvm(dut = SyncFifo)]
pub struct FifoTransaction {
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
    const fn reset() -> Self {
        Self { reset_n: false, push: false, push_data: 0, pop: false }
    }

    const fn operation(push: bool, push_data: u8, pop: bool) -> Self {
        Self { reset_n: true, push, push_data, pop }
    }

    const fn operation_kind(self) -> FifoOperation {
        match (self.push, self.pop) {
            (false, false) => FifoOperation::Idle,
            (true, false) => FifoOperation::Push,
            (false, true) => FifoOperation::Pop,
            (true, true) => FifoOperation::Simultaneous,
        }
    }
}

/// Sampled outputs after one clock edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FifoObservation {
    pop_data: u8,
    occupancy: u8,
    boundary: Boundary,
    acceptance: Acceptance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Boundary {
    Empty,
    Available,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Acceptance {
    Accepted,
    Overflow,
    Underflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FifoOperation {
    Idle,
    Push,
    Pop,
    Simultaneous,
}

impl Sample<SyncFifo> for FifoObservation {
    fn sample(dut: &SyncFifo) -> std::result::Result<Self, SyncFifoError> {
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

        Ok(Self { pop_data: dut.pop_data()?, occupancy, boundary, acceptance })
    }
}

#[derive(Default)]
struct FifoModel {
    queue: VecDeque<u8>,
    pop_data: u8,
}

impl ReferenceModel<FifoTransaction> for FifoModel {
    type Expected = FifoObservation;

    fn predict(&mut self, transaction: &FifoTransaction) -> Self::Expected {
        if !transaction.reset_n {
            self.queue.clear();
            self.pop_data = 0;
            return self.observation(false, false);
        }

        let was_empty = self.queue.is_empty();
        let was_full = self.queue.len() == DEPTH;
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

/// Replayable deterministic sequence.
pub struct RandomSequence {
    random: RandomContext,
    remaining: u64,
    reset_pending: bool,
}

impl RandomSequence {
    pub fn new(replay: ReplayToken, cycles: u64) -> Self {
        Self { random: RandomContext::from_replay(replay), remaining: cycles, reset_pending: cycles != 0 }
    }
}

impl ReplayableSequence for RandomSequence {
    fn replay_token(&self) -> ReplayToken {
        self.random.replay_token()
    }
}

impl Iterator for RandomSequence {
    type Item = FifoTransaction;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining = self.remaining.checked_sub(1)?;

        if self.reset_pending {
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

/// Manual benchmark coverage model.
pub struct FifoCoverage {
    operation: Coverpoint<FifoOperation>,
    boundary: Coverpoint<Boundary>,
    operation_x_boundary: Cross2,
}

impl FifoCoverage {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let operation = Coverpoint::builder("operation")
            .bin(Bin::value("idle", FifoOperation::Idle))
            .bin(Bin::value("push", FifoOperation::Push))
            .bin(Bin::value("pop", FifoOperation::Pop))
            .bin(Bin::value("simultaneous", FifoOperation::Simultaneous))
            .build()?;
        let boundary = Coverpoint::builder("boundary")
            .bin(Bin::value("empty", Boundary::Empty))
            .bin(Bin::value("available", Boundary::Available))
            .bin(Bin::value("full", Boundary::Full))
            .build()?;
        let operation_x_boundary = Cross2::builder("operation_x_boundary", &operation, &boundary).build()?;

        Ok(Self { operation, boundary, operation_x_boundary })
    }

    pub fn sample(&mut self, cycle: ObservedCycle<'_, FifoTransaction, FifoObservation>) -> Result<(), Box<dyn std::error::Error>> {
        let operation = self.operation.sample(&cycle.stimulus().operation_kind())?;
        let boundary = self.boundary.sample(&cycle.observed().boundary)?;
        self.operation_x_boundary.sample(&operation, &boundary)?;
        Ok(())
    }
}

/// Runs the normal FIFO testbench without manual coverage sampling.
pub fn run_testbench() -> Result<FifoResult, SyncFifoError> {
    let dut = SyncFifo::new()?;

    run_sequence(dut, RandomSequence::new(BENCH_REPLAY, BENCH_CYCLES))
}

/// Runs a deterministic push-heavy workload.
pub fn run_push_only() -> Result<FifoResult, SyncFifoError> {
    let dut = SyncFifo::new()?;
    let sequence = std::iter::once(FifoTransaction::reset()).chain(
        (0..BENCH_CYCLES.saturating_sub(1)).map(|index| {
            FifoTransaction::operation(true, u8::try_from(index & 0xff).unwrap_or_default(), false)
        }),
    );

    run_sequence(dut, sequence)
}

/// Runs a deterministic pop-heavy workload.
pub fn run_pop_only() -> Result<FifoResult, SyncFifoError> {
    let dut = SyncFifo::new()?;
    let sequence = std::iter::once(FifoTransaction::reset()).chain(
        (0..BENCH_CYCLES.saturating_sub(1)).map(|index| {
            if index < 16 {
                FifoTransaction::operation(true, u8::try_from(index & 0xff).unwrap_or_default(), false)
            } else {
                FifoTransaction::operation(false, 0, true)
            }
        }),
    );

    run_sequence(dut, sequence)
}

/// Runs a deterministic simultaneous push/pop workload.
pub fn run_simultaneous() -> Result<FifoResult, SyncFifoError> {
    let dut = SyncFifo::new()?;
    let sequence = std::iter::once(FifoTransaction::reset()).chain(
        (0..BENCH_CYCLES.saturating_sub(1)).map(|index| {
            let value = u8::try_from(index & 0xff).unwrap_or_default();
            let priming = index < 8;
            FifoTransaction::operation(true, value, !priming)
        }),
    );

    run_sequence(dut, sequence)
}

fn run_sequence(
    dut: SyncFifo,
    sequence: impl Iterator<Item = FifoTransaction>,
) -> Result<FifoResult, SyncFifoError> {
    Ok(Testbench::new(dut)
        .with_sequence(sequence)
        .with_reference_model(FifoModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(FifoClock)
        .run::<FifoObservation>())
}

/// Runs the normal FIFO testbench while sampling manual coverage.
pub fn run_testbench_with_coverage() -> Result<FifoResult, Box<dyn std::error::Error>> {
    let dut = SyncFifo::new()?;
    let mut coverage = FifoCoverage::new()?;

    let result = Testbench::new(dut)
        .with_sequence(RandomSequence::new(BENCH_REPLAY, BENCH_CYCLES))
        .with_reference_model(FifoModel::default())
        .with_scoreboard(ExactScoreboard)
        .with_clock(FifoClock)
        .run_with_observer::<FifoObservation, _>(|cycle| {
            let sampled = coverage.sample(cycle);
            assert!(sampled.is_ok(), "sync-fifo coverage sampling must succeed during benches");
        });

    Ok(result)
}

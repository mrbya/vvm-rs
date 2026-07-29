//! Directed asynchronous FIFO verification scenarios.

use vvm::coverage::{
    Bin, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor, CoverageItemRef, Coverpoint,
};
use vvm::dut::Dut;
use vvm::random::{RandomContext, ReplayToken, Seed};
use vvm::test::TestContext;
use vvm::testbench::TestResult;
use vvm::timing::SimulationTime;

use crate::async_fifo::AsyncFifo;
use crate::verification::{
    QueueModel, Result, read_edge, reset, synchronize_read_domain, synchronize_write_domain,
    write_edge,
};

/// Stable random stream for the direct multi-clock randomized scenario.
const RANDOM_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0xa5_79_c1_02));

/// Accepted or rejected transfer kind recorded by async-FIFO coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    WriteAccepted,
    WriteRejected,
    ReadAccepted,
    ReadRejected,
}

/// Data-value grouping used by async-FIFO coverage.
/// Logical queue region sampled after a transfer attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataClass {
    Zero,
    Maximum,
    Alternating,
    Other,
}

/// Logical queue region sampled after a transfer attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Occupancy {
    Empty,
    Available,
    Full,
}

/// Manual coverage sampled from directly driven FIFO transfers.
struct AsyncFifoCoverage {
    instance: CoverageGroupInstance,
    operation: Coverpoint<Operation>,
    data_class: Coverpoint<DataClass>,
    occupancy: Coverpoint<Occupancy>,
}

impl AsyncFifoCoverage {
    /// Builds transfer, data-class, and logical-occupancy coverpoints.
    fn new() -> Result<Self> {
        let operation = Coverpoint::builder("operation")
            .bin(Bin::value("write_accepted", Operation::WriteAccepted))
            .bin(Bin::value("write_rejected", Operation::WriteRejected))
            .bin(Bin::value("read_accepted", Operation::ReadAccepted))
            .bin(Bin::value("read_rejected", Operation::ReadRejected))
            .build()?;
        let data_class = Coverpoint::builder("data_class")
            .bin(Bin::value("zero", DataClass::Zero))
            .bin(Bin::value("maximum", DataClass::Maximum))
            .bin(Bin::value("alternating", DataClass::Alternating))
            .bin(Bin::value("other", DataClass::Other))
            .build()?;
        let occupancy = Coverpoint::builder("occupancy")
            .bin(Bin::value("empty", Occupancy::Empty))
            .bin(Bin::value("available", Occupancy::Available))
            .bin(Bin::value("full", Occupancy::Full))
            .build()?;

        Ok(Self {
            instance: CoverageGroupInstance::new("async_fifo", "dut.async_fifo")?,
            operation,
            data_class,
            occupancy,
        })
    }

    /// Records one write request and its acceptance against the queue model.
    fn sample_write(&mut self, value: u8, accepted: bool, occupancy: usize) -> Result<()> {
        let operation = if accepted {
            Operation::WriteAccepted
        } else {
            Operation::WriteRejected
        };

        self.operation.sample(&operation)?;
        self.data_class.sample(&classify_data(value))?;
        self.occupancy.sample(&classify_occupancy(occupancy))?;

        Ok(())
    }

    /// Records one read request and its acceptance against the queue model.
    fn sample_read(&mut self, accepted: bool, occupancy: usize) -> Result<()> {
        let operation = if accepted {
            Operation::ReadAccepted
        } else {
            Operation::ReadRejected
        };

        self.operation.sample(&operation)?;
        self.occupancy.sample(&classify_occupancy(occupancy))?;

        Ok(())
    }
}

impl CoverageGroup for AsyncFifoCoverage {
    /// Returns the stable asynchronous-FIFO coverage identity.
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    /// Visits coverage items in deterministic artifact order.
    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        visitor.visit(CoverageItemRef::coverpoint(&self.operation));
        visitor.visit(CoverageItemRef::coverpoint(&self.data_class));
        visitor.visit(CoverageItemRef::coverpoint(&self.occupancy));
    }
}

/// Groups generated data values into coverage classes.
const fn classify_data(value: u8) -> DataClass {
    match value {
        0 => DataClass::Zero,
        u8::MAX => DataClass::Maximum,
        0x55 | 0xaa => DataClass::Alternating,
        _ => DataClass::Other,
    }
}

/// Converts logical queue length into empty, available, or full coverage bins.
const fn classify_occupancy(occupancy: usize) -> Occupancy {
    match occupancy {
        0 => Occupancy::Empty,
        8 => Occupancy::Full,
        _ => Occupancy::Available,
    }
}

/// Applies a list of writes while retaining only accepted values in the model.
fn write_values(dut: &mut AsyncFifo, model: &mut QueueModel, values: &[u8]) -> Result<()> {
    for value in values {
        let accepted = !dut.full()?;
        write_edge(dut, true, *value)?;
        if accepted {
            model.write(*value);
        }
    }

    Ok(())
}

/// Applies read attempts and compares every accepted value with queue order.
fn read_values(dut: &mut AsyncFifo, model: &mut QueueModel, reads: usize) -> Result<()> {
    for _ in 0..reads {
        let observed = read_edge(dut, true)?;
        if let Some(observed) = observed {
            assert_eq!(Some(observed), model.read());
        }
    }

    Ok(())
}

/// Drives and covers one write-domain transfer attempt.
fn covered_write(
    dut: &mut AsyncFifo,
    model: &mut QueueModel,
    coverage: &mut AsyncFifoCoverage,
    value: u8,
) -> Result<()> {
    let accepted = !dut.full()?;
    write_edge(dut, true, value)?;

    if accepted {
        model.write(value);
    }

    coverage.sample_write(value, accepted, model.len())?;

    Ok(())
}

/// Drives and covers one read-domain transfer attempt.
fn covered_read(
    dut: &mut AsyncFifo,
    model: &mut QueueModel,
    coverage: &mut AsyncFifoCoverage,
) -> Result<()> {
    let observed = read_edge(dut, true)?;
    if let Some(observed) = observed {
        assert_eq!(Some(observed), model.read());
    }

    coverage.sample_read(observed.is_some(), model.len())?;

    Ok(())
}

/// Fills then drains the FIFO without losing data order.
#[test]
fn async_fifo_fill_and_drain() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    write_values(&mut dut, &mut model, &[0x10, 0x11, 0x12, 0x13])?;
    synchronize_read_domain(&mut dut)?;
    read_values(&mut dut, &mut model, 8)?;

    assert_eq!(model.len(), 0);
    assert!(dut.empty()?);
    Ok(())
}

/// Exercises a faster write domain before the read synchronizer catches up.
#[test]
fn async_fifo_write_faster_than_read() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    write_values(&mut dut, &mut model, &[1, 2, 3, 4, 5, 6])?;
    synchronize_read_domain(&mut dut)?;
    read_values(&mut dut, &mut model, 10)?;

    assert!(model.len() == 0);
    Ok(())
}

/// Fills the FIFO to full, rejects one additional write, then drains in order.
#[test]
fn async_fifo_full_behavior() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    write_values(&mut dut, &mut model, &[0, 1, 2, 3, 4, 5, 6, 7])?;
    assert!(dut.full()?);
    write_values(&mut dut, &mut model, &[0xff])?;
    assert_eq!(model.len(), 8);

    synchronize_read_domain(&mut dut)?;
    read_values(&mut dut, &mut model, 12)?;
    assert_eq!(model.len(), 0);
    Ok(())
}

/// Exercises a faster read domain and verifies empty rejection before writes arrive.
#[test]
fn async_fifo_read_faster_than_write() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    assert_eq!(read_edge(&mut dut, true)?, None);
    write_values(&mut dut, &mut model, &[0xa0, 0xa1, 0xa2])?;
    synchronize_read_domain(&mut dut)?;
    read_values(&mut dut, &mut model, 7)?;

    assert_eq!(model.len(), 0);
    Ok(())
}

/// Forces pointer wraparound while preserving queue order.
#[test]
fn async_fifo_wraparound() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    for value in 0..16_u8 {
        write_values(&mut dut, &mut model, &[value])?;
        synchronize_read_domain(&mut dut)?;
        read_values(&mut dut, &mut model, 1)?;
        synchronize_write_domain(&mut dut)?;
    }

    assert_eq!(model.len(), 0);
    Ok(())
}

/// Applies both edges at one logical timestamp in a deterministic write-then-read order.
#[test]
fn async_fifo_coincident_edges() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    write_values(&mut dut, &mut model, &[0x5a])?;
    synchronize_read_domain(&mut dut)?;
    read_values(&mut dut, &mut model, 1)?;

    assert_eq!(model.len(), 0);
    Ok(())
}

/// Runs reproducible pseudo-random traffic through repeated domain synchronization.
#[vvm::test(replay(default = RANDOM_REPLAY), coverage)]
fn async_fifo_random(
    context: &mut TestContext,
) -> std::result::Result<
    TestResult<(), String, crate::async_fifo::AsyncFifoError>,
    crate::verification::Error,
> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    let replay = context.config().replay_token_or(RANDOM_REPLAY);
    let mut random = RandomContext::from_replay(replay);
    let mut coverage = AsyncFifoCoverage::new()?;

    for value in [0, u8::MAX, 0x55, 0xaa, 1, 2, 3, 4] {
        covered_write(&mut dut, &mut model, &mut coverage, value)?;
    }
    covered_write(&mut dut, &mut model, &mut coverage, 0xff)?;
    synchronize_read_domain(&mut dut)?;

    for _ in 0..9 {
        covered_read(&mut dut, &mut model, &mut coverage)?;
    }
    synchronize_write_domain(&mut dut)?;

    for _ in 0..32_u8 {
        let value = u8::try_from(random.next_u32() & 0xff).unwrap_or_default();
        covered_write(&mut dut, &mut model, &mut coverage, value)?;
        synchronize_read_domain(&mut dut)?;
        covered_read(&mut dut, &mut model, &mut coverage)?;
        synchronize_write_domain(&mut dut)?;
    }

    assert_eq!(model.len(), 0);
    let final_time = dut.simulation_time();
    dut.finish()?;
    context.capture_coverage(&coverage)?;

    Ok(TestResult::completed(
        SimulationTime::ZERO,
        final_time,
        Some(replay),
    ))
}

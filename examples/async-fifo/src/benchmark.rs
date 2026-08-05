//! Hidden native benchmark harness for the asynchronous FIFO example.

use vvm::random::{RandomContext, ReplayToken, Seed};

use crate::async_fifo::AsyncFifo;
use crate::verification::{
    QueueModel, Result, read_edge, reset, synchronize_read_domain, synchronize_write_domain,
    write_edge,
};

/// Stable random stream for the async benchmark traffic.
pub const BENCH_REPLAY: ReplayToken = ReplayToken::new(Seed::new(0xa5_79_c1_02));
/// Number of transfer rounds used by the benchmark workload.
pub const BENCH_ROUNDS: u32 = 64;

/// Runs deterministic async-FIFO traffic with synchronization between domains.
pub fn run_randomized() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    let mut random = RandomContext::from_replay(BENCH_REPLAY);

    for seed_value in [0_u8, u8::MAX, 0x55, 0xaa, 1, 2, 3, 4] {
        let accepted = !dut.full()?;
        write_edge(&mut dut, true, seed_value)?;
        if accepted {
            model.write(seed_value);
        }
    }

    synchronize_read_domain(&mut dut)?;

    for _ in 0..BENCH_ROUNDS {
        let write_value = u8::try_from(random.next_u32() & 0xff).unwrap_or_default();
        let accepted_write = !dut.full()?;
        write_edge(&mut dut, true, write_value)?;
        if accepted_write {
            model.write(write_value);
        }

        synchronize_read_domain(&mut dut)?;

        let observed = read_edge(&mut dut, true)?;
        if let Some(observed) = observed {
            assert_eq!(Some(observed), model.read());
        }

        synchronize_write_domain(&mut dut)?;
    }

    let remaining = model.len();
    assert!(remaining <= 8, "logical async-fifo occupancy must stay within depth");

    Ok(())
}

/// Runs the deterministic fill-and-drain scenario.
pub fn run_fill_and_drain() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    for value in [0x10_u8, 0x11, 0x12, 0x13] {
        write_edge(&mut dut, true, value)?;
        model.write(value);
    }

    synchronize_read_domain(&mut dut)?;

    for _ in 0..8 {
        let observed = read_edge(&mut dut, true)?;
        if let Some(observed) = observed {
            assert_eq!(Some(observed), model.read());
        }
    }

    Ok(())
}

/// Runs a deterministic coincident-edge style sequence.
pub fn run_coincident_edges() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    write_edge(&mut dut, true, 0x5a)?;
    model.write(0x5a);
    synchronize_read_domain(&mut dut)?;

    let observed = read_edge(&mut dut, true)?;
    if let Some(observed) = observed {
        assert_eq!(Some(observed), model.read());
    }

    Ok(())
}

/// Runs a write-domain-faster-than-read scenario.
pub fn run_write_faster_than_read() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    for value in [1_u8, 2, 3, 4, 5, 6] {
        write_edge(&mut dut, true, value)?;
        model.write(value);
    }

    synchronize_read_domain(&mut dut)?;

    for _ in 0..10 {
        let observed = read_edge(&mut dut, true)?;
        if let Some(observed) = observed {
            assert_eq!(Some(observed), model.read());
        }
    }

    Ok(())
}

/// Runs a read-domain-faster-than-write scenario.
pub fn run_read_faster_than_write() -> Result<()> {
    let mut dut = AsyncFifo::new()?;
    let mut model = QueueModel::default();
    reset(&mut dut)?;

    assert_eq!(read_edge(&mut dut, true)?, None);

    for value in [0xa0_u8, 0xa1, 0xa2] {
        write_edge(&mut dut, true, value)?;
        model.write(value);
    }

    synchronize_read_domain(&mut dut)?;

    for _ in 0..7 {
        let observed = read_edge(&mut dut, true)?;
        if let Some(observed) = observed {
            assert_eq!(Some(observed), model.read());
        }
    }

    Ok(())
}

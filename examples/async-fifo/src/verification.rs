//! Direct multi-clock operations for the asynchronous FIFO example.

use std::collections::VecDeque;

use thiserror::Error;
use vvm::dut::Dut;
use vvm::timing::TimeStep;

use crate::async_fifo::{AsyncFifo, AsyncFifoError};

/// Asynchronous FIFO example error.
#[derive(Debug, Error)]
pub enum Error {
    /// Generated DUT operation failed.
    #[error(transparent)]
    Dut(#[from] AsyncFifoError),
    /// Functional-coverage definition could not be built.
    #[error(transparent)]
    CoverageBuild(#[from] vvm::coverage::BuildError),
    /// Functional-coverage group could not be created.
    #[error(transparent)]
    CoverageGroup(#[from] vvm::coverage::GroupError),
    /// Functional-coverage sample could not be recorded.
    #[error(transparent)]
    CoverageSample(#[from] vvm::coverage::SampleError),
    /// Functional-coverage session could not capture the group.
    #[error(transparent)]
    CoverageSession(#[from] vvm::coverage::session::SessionError),
}

/// Asynchronous FIFO example result.
pub type Result<T> = std::result::Result<T, Error>;

/// Logical queue model used to check accepted FIFO data transfers.
#[derive(Debug, Default)]
pub struct QueueModel {
    /// Expected data in FIFO order.
    values: VecDeque<u8>,
}

impl QueueModel {
    /// Records one accepted write.
    pub fn write(&mut self, value: u8) {
        self.values.push_back(value);
    }

    /// Returns the next expected accepted read value.
    pub fn read(&mut self) -> Option<u8> {
        self.values.pop_front()
    }

    /// Returns the current logical occupancy.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

/// Initializes both clock domains at time zero.
pub fn reset(dut: &mut AsyncFifo) -> Result<()> {
    dut.set_wr_clk(false)?;
    dut.set_rd_clk(false)?;
    dut.set_write(false)?;
    dut.set_read(false)?;
    // Drive a high-to-low transition so an asynchronously reset DUT receives
    // a real negedge even when generated inputs initially default to low.
    dut.set_wr_reset_n(true)?;
    dut.set_rd_reset_n(true)?;
    dut.eval()?;

    dut.set_wr_reset_n(false)?;
    dut.set_rd_reset_n(false)?;
    dut.eval()?;

    dut.set_wr_reset_n(true)?;
    dut.set_rd_reset_n(true)?;
    dut.eval()?;

    Ok(())
}

/// Drives one write-domain edge and advances deterministic simulation time.
pub fn write_edge(dut: &mut AsyncFifo, write: bool, value: u8) -> Result<()> {
    dut.set_write(write)?;
    dut.set_write_data(value)?;
    dut.set_wr_clk(false)?;
    dut.eval()?;

    // Only this rising edge mutates the write-domain pointer and memory.
    dut.set_wr_clk(true)?;
    dut.eval()?;

    dut.set_wr_clk(false)?;
    dut.eval()?;
    Dut::advance_time(dut, TimeStep::ONE)?;

    Ok(())
}

/// Drives one read-domain edge and returns an accepted read value when present.
pub fn read_edge(dut: &mut AsyncFifo, read: bool) -> Result<Option<u8>> {
    // Capture the pre-edge flag: read_data is valid only for an accepted read.
    let was_empty = dut.empty()?;
    dut.set_read(read)?;
    dut.set_rd_clk(false)?;
    dut.eval()?;

    dut.set_rd_clk(true)?;
    dut.eval()?;

    let value = (read && !was_empty).then(|| dut.read_data()).transpose()?;
    dut.set_rd_clk(false)?;
    dut.eval()?;
    Dut::advance_time(dut, TimeStep::ONE)?;

    Ok(value)
}

/// Advances the read synchronizer without consuming data.
pub fn synchronize_read_domain(dut: &mut AsyncFifo) -> Result<()> {
    // Two synchronizer registers plus the registered empty calculation require
    // three idle read edges before a newly written pointer is observable.
    read_edge(dut, false)?;
    read_edge(dut, false)?;
    read_edge(dut, false)?;

    Ok(())
}

/// Advances the write synchronizer without accepting data.
pub fn synchronize_write_domain(dut: &mut AsyncFifo) -> Result<()> {
    // The write side has the symmetric latency for the returned read pointer.
    write_edge(dut, false, 0)?;
    write_edge(dut, false, 0)?;
    write_edge(dut, false, 0)?;

    Ok(())
}

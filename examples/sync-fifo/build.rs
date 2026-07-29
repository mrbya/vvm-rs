//! Builds the synchronous FIFO model and VCD-capable bridge.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

/// Generates and compiles the trace-capable FIFO wrapper.
fn main() -> BuildResult<()> {
    DutBuilder::new("sync_fifo")
        .top_module("sync_fifo")
        .source("rtl/sync_fifo.sv")
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}

//! Builds the generated C++ adapter and Verilated asynchronous FIFO model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

/// Generates and compiles the trace-capable asynchronous FIFO wrapper.
fn main() -> BuildResult<()> {
    DutBuilder::new("async_fifo")
        .top_module("async_fifo")
        .source("rtl/async_fifo.sv")
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}

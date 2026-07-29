//! Builds the behavioral timing-enabled UART model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

/// Generates and compiles the timing-enabled behavioral UART wrapper.
fn main() -> BuildResult<()> {
    DutBuilder::new("timed_uart")
        .top_module("timed_uart")
        .source("rtl/timed_uart.sv")
        .timing()
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}

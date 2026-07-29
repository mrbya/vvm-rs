//! Builds the synthetic independent-clock scheduler regression model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("multi_clock_counter")
        .top_module("multi_clock_counter")
        .source("rtl/multi_clock_counter.sv")
        .trace(TraceOptions::vcd())
        .build()
}

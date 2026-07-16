//! Builds the generated C++ adapter and Verilated multi-clock model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("multi_clock_counter")
        .top_module("multi_clock_counter")
        .source("rtl/multi_clock_counter.sv")
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}

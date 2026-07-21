//! Builds the tri-state bus example DUT.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("tri_state_bus")
        .top_module("tri_state_bus")
        .source("rtl/tri_state_bus.sv")
        .trace(TraceOptions::vcd().with_depth(10))
        .build()
}

//! Builds the timing-enabled delayed-event model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("delayed_sequence")
        .top_module("delayed_sequence")
        .source("rtl/delayed_sequence.sv")
        .timing()
        .trace(TraceOptions::vcd().with_depth(10))
        .build()
}

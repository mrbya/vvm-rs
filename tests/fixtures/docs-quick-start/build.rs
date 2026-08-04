// ANCHOR: build-script
use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("event_counter")
        .top_module("event_counter")
        .source("rtl/event_counter.sv")
        .trace(TraceOptions::vcd().with_depth(8))
        .build()
}
// ANCHOR_END: build-script

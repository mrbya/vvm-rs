//! Builds the generated C++ adapter, handwritten CXX bridge, and
//! Verilated counter model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}

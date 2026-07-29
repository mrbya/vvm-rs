//! Builds the generated C++ adapter and Verilated wide-transform model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("wide_transform")
        .top_module("wide_transform")
        .source("rtl/wide_transform.sv")
        .trace(TraceOptions::vcd().with_depth(20))
        .build()
}

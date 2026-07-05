//! Builds the generated C++ adapter and Verilated signed-adder model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("signed_adder")
        .top_module("signed_adder")
        .source("rtl/signed_adder.sv")
        .trace(TraceOptions::vcd().with_depth(20))
        .build()
}

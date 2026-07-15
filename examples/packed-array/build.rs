//! Builds the generated C++ adapter and Verilated packed-array example model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("packed_array_ports")
        .top_module("packed_array_ports")
        .source("rtl/packed_array_ports.sv")
        .trace(TraceOptions::vcd().with_depth(20))
        .build()
}

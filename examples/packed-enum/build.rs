//! Builds the generated C++ adapter and Verilated packed-enum example model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("packed_enum_ports")
        .top_module("packed_enum_ports")
        .source("rtl/packed_enum_ports.sv")
        .trace(TraceOptions::vcd().with_depth(20))
        .build()
}

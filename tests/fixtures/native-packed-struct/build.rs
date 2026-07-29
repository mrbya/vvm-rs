//! Builds the generated C++ adapter and Verilated packed-struct example model.

use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("packed_struct_ports")
        .top_module("packed_struct_ports")
        .source("rtl/packed_struct_ports.sv")
        .trace(TraceOptions::vcd().with_depth(20))
        .build()
}

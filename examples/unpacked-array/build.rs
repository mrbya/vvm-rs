//! Builds the generated C++ adapter and Verilated unpacked-array model.

use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("unpacked_array_ports")
        .top_module("unpacked_array_ports")
        .source("rtl/unpacked_array_ports.sv")
        .build()
}

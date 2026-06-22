//! Builds the generated C++ adapter, handwritten CXX bridge, and
//! Verilated counter model.

use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .build()
}

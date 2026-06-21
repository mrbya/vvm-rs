//! Builds the handwritten CXX bridge and Verilated counter model.

use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .bridge("src/bridge.rs")
        .cpp_source("cpp/counter.cpp")
        .cpp_include("cpp")
        .build()
}

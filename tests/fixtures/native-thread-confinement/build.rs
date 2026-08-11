use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .build()
}

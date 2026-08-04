use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("line_adapter")
        .top_module("line_adapter")
        .source("rtl/line_adapter.sv")
        .build()
}

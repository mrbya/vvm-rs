# Getting started

VVM targets Linux development and CI environments with Rust 1.87.0 or newer, a C++17 compiler, and Verilator. Timing-enabled models additionally need compiler coroutine support. CI verifies the minimum supported Verilator 5.000 and current tested Verilator 5.050. Other platforms and Verilator releases are not currently a support guarantee.

Install `just` for repository workflows. A consumer uses `vvm-rs` as `vvm` and `vvm-build` in `build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }

[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

The maintained [counter example](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/counter) is the complete first project. Its layout is `Cargo.toml`, `rtl/counter.sv`, `build.rs`, and `src/lib.rs`. The build script is intentionally small:

```rust,no_run
use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .trace(TraceOptions::vcd())
        .build()
}
```

The test module includes the generated wrapper with `vvm::include_dut!(counter)`, derives `Drive`, `Sample`, and `Clock` for stimulus, observation, and clock types, and constructs a `Testbench` with an iterator sequence, reference model, and exact scoreboard. Run `cargo test counter_smoke`; Cargo reports an ordinary passing Rust test. Use `VERILATOR=/path/to/verilator` when it is not on `PATH`. Build failures identify their Verilator stage and command; confirm HDL source paths are relative to the consumer manifest.

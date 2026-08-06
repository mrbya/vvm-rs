# vvm-build

`vvm-build` is the build-script crate that runs Verilator, generates the bridge,
and compiles the native support code for a VVM DUT wrapper.

Add it to `[build-dependencies]`:

```toml
[build-dependencies]
vvm-build = "0.2.0"
```

## Who Should Use It?

Use `vvm-build` when you are writing a consumer crate that needs a generated DUT
wrapper at build time. Most users pair it with `vvm-rs` in test code.

## Minimal `build.rs`

```rust
use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .build()
}
```

The logical name passed to `DutBuilder::new("counter")` must match the name
used later by `vvm::include_dut!(counter)`.

## Normal Workflow

`DutBuilder` configures:

- the logical DUT name
- the top module
- HDL source files
- include directories
- tracing support
- timing-enabled model generation

The generated wrapper is written under `OUT_DIR` and is included by the consumer
crate; it is not meant to be edited directly.

## Common Configuration Areas

- Use `.source(...)` for HDL files.
- Use `.top_module(...)` when the Verilator top is not implicit.
- Enable tracing when tests need VCD output.
- Enable timing mode when the DUT uses delayed HDL events that must be exposed to
  Rust.

If Verilator is not on `PATH`, set `VERILATOR=/path/to/verilator`.

## Build Pipeline Summary

`vvm-build` performs this pipeline:

1. validate builder configuration
2. invoke Verilator
3. parse and normalize metadata
4. map HDL shapes to wrapper types
5. generate the adapter and bridge code
6. compile the native support code

`BuildError` retains the failing `BuildStage` so diagnostics identify which step
failed.

## Common Failure Cases

- logical DUT name does not match `include_dut!`
- bad HDL source path
- missing Verilator executable
- unsupported or unexpected generated port shape
- timing mode requested without the required native compiler support

## Documentation

- Build guide: <https://byacrates.gitlab.io/vvm-rs/guide/build-script.html>
- Including the DUT: <https://byacrates.gitlab.io/vvm-rs/guide/including-the-dut.html>
- Timing guide: <https://byacrates.gitlab.io/vvm-rs/guide/timing-models.html>
- API reference: <https://byacrates.gitlab.io/vvm-rs/api/vvm_build/>
- Counter example: <https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/counter>

# VVM

Rust-first verification for Verilator-generated HDL models.

VVM gives a Rust crate a generated DUT wrapper, typed drive/sample APIs,
testbench composition, replayable randomization, waveform tracing, timing-mode
support for delayed HDL events, and Rust-native functional coverage.

> [!WARNING]
> VVM is an early alpha. The core workflow is real and CI-tested, but public APIs
> may still change before the first stable release.

## What Is VVM?

VVM wraps Verilator-generated simulation models of RTL designs in a Rust verification workflow.

VVM is for engineers who want typed verification logic in ordinary Rust tests.
It is not a four-state simulator, a UVM compatibility layer, or a CDC proof
tool.

## Why VVM?

- Keep verification code in normal Rust crates and normal Cargo workflows.
- Use typed transactions, models, scoreboards, and coverage instead of ad hoc
  stringly scripts.
- Reproduce randomized failures with replay tokens.
- Use the same public workflow from the smallest counter example through timed,
  multi-clock, and inout-oriented examples.

## Current Status

- Linux-focused native verification support.
- Two-state Verilator behavior: Rust-visible ports do not carry HDL `X` or `Z`.
- Timing mode supports delayed future slots, not same-time or `#0` scheduling.

## Features

- Verilator-backed generated DUT wrappers.
- `Drive`, `Sample`, and `Clock` derives.
- `Testbench`, reference models, and scoreboards.
- Replayable randomization.
- VCD tracing.
- Timing-mode scheduling for delayed HDL behavior.
- Deterministic multi-clock workflows.
- Caller-owned inout resolution.
- Rust-native functional coverage and offline reporting.

## Requirements

- Rust 1.87.0 or newer.
- Verilator.
- A working Linux C++ toolchain.

Timing-enabled models additionally need coroutine-capable C++ support.

## Installation

Most users add the facade crate as `vvm` and use `vvm-build` in `build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }

[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

Install `cargo-vvm` when you want suite-level functional-coverage merge and
reporting:

```bash
cargo install cargo-vvm
```

## Crate layout

- `vvm-build` runs Verilator during `build.rs`, generates rust-to-c++ bridges, and compiles rust-native native wrappers.
- `vvm` is the public VVM library entrypoint.
- `cargo-vvm` is a cargo utility cli tool to manage functional coverage artifacts and reports.

## Quickstart

Minimal project example:

```text
counter-verification/
├── Cargo.toml
├── build.rs        # vvm-build Verilator bootstrap
├── rtl/
│   └── counter.sv  # your design HDL source
└── src/
    └── lib.rs      # vvm rust testbench and test setup
```

`rtl/counter.sv`:
```systemverilog
module counter (
    input  logic       clk,
    input  logic       reset_n,
    input  logic       enable,
    output logic [7:0] count
);

always_ff @(posedge clk or negedge reset_n) begin
    if (!reset_n) begin
        count <= '0;
    end else if (enable) begin
        count <= count + 1'b1;
    end
end

endmodule
```

`build.rs`:

```rust
use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .trace(TraceOptions::vcd())
        .build()
}
```

`src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use vvm::prelude::*;

    vvm::include_dut!(counter);

    use crate::counter::{Counter, CounterError};

    /// Stimulus definition.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Drive)]
    #[vvm(dut = Counter)]
    struct Stimulus {
        #[vvm(port)]
        reset_n: bool,
        #[vvm(port)]
        enable: bool,
    }

    /// Sample definition.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Sample)]
    #[vvm(dut = Counter)]
    struct Observation {
        #[vvm(port)]
        count: u8,
    }

    /// Clocking signal definition.
    #[derive(Clone, Copy, Debug, Default, Clock)]
    #[vvm(dut = Counter, clock = "clk")]
    struct CounterClock;

    /// Test result type alias.
    type CounterTestResult = TestResult<
        CounterStimulus,
        Mismatch<CounterObservation, CounterObservation>,
        CounterError,
    >;

    /// Counter smoke verification.
    #[vvm::test(trace)]
    fn counter_smoke(config: &vvm::TestRunConfig) -> Result<CounterTestResult, counter::CounterError> {
        let mut dut = Counter::new()?;

        config.configure_trace(&mut dut)?;

        Ok(
            Testbench::new(dut)
                // Add a sequence, reference model, scoreboard, and clock.
                .run::<CounterObservation>(),
        )
    }
}
```

Run tests with normal Cargo commands:

```bash
cargo test
# or
cargo nextest run
```

## Example Ladder

- `examples/counter`: first complete workflow.
- `examples/sync-fifo`: queue model and boundary behavior.
- `examples/timed-uart`: timing scheduler and protocol reconstruction.
- `examples/async-fifo`: deterministic multi-clock workflow.
- `examples/tri-state-bus`: caller-owned inout resolution.

## Development

Repository contributors should start with:

```bash
cargo install just
just init
```

Common commands:

```bash
just fmt --check
just check -- -D warnings
just test-fast
just test-native
just docs-test
just ci
```

## Documentation

- Project book: <https://byacrates.gitlab.io/vvm-rs/>
- Quick start: <https://byacrates.gitlab.io/vvm-rs/quick-start.html>
- API reference: <https://byacrates.gitlab.io/vvm-rs/api/>
- Example ladder: <https://byacrates.gitlab.io/vvm-rs/examples.html>

## Similar Projects And Inspiration

VVM is heavily inspired by UVM-style verification structure and by the original
VVM reference project preserved under `docs/dev/reference-projects/vvm/`.

## License

Licensed under either of the following, at your option:

- Apache License 2.0 (`LICENSE-APACHE`)
- MIT (`LICENSE-MIT`)

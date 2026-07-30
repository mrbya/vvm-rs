# Quick Start

This quick start walks through the maintained counter example workflow. It uses
real VVM code and real example sources, not pseudocode.

## 1. Check Prerequisites

Confirm that Rust, Cargo, and Verilator are installed:

```bash
rustc --version
cargo --version
verilator --version
```

## 2. Create A Verification Project

Create a library crate so your tests live next to the generated DUT wrapper:

```bash
cargo new counter-verification --lib
```

Use this layout:

```text
counter-verification/
├── Cargo.toml
├── build.rs
├── rtl/
│   └── counter.sv
└── src/
    └── lib.rs
```

## 3. Add Dependencies

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }

[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

## 4. Add A Small DUT

Use the maintained counter RTL from `examples/counter/rtl/counter.sv` or copy an
equivalent module into `rtl/counter.sv`.

The DUT has four behaviors you care about in this quick start:

- `reset_n = 0` clears the count.
- `enable = 1` increments the count.
- `enable = 0` holds the current value.
- the visible state updates on the rising edge of `clk`.

## 5. Create `build.rs`

The counter example build script is small enough to reuse directly:

```rust
use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .trace(TraceOptions::vcd().with_depth(30))
        .build()
}
```

The logical DUT name passed to `DutBuilder::new("counter")` must match the name
used later in `vvm::include_dut!(counter)`.

## 6. Include The Generated DUT

In `src/lib.rs`, include the generated wrapper inside a test-only module:

```rust
#[cfg(test)]
mod tests {
    vvm::include_dut!(counter);
}
```

## 7. Define Stimulus And Observation

The maintained counter example defines typed inputs and outputs with derives:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, vvm::Drive)]
#[vvm(dut = crate::counter::Counter)]
pub struct CounterStimulus {
    #[vvm(port)]
    reset_n: bool,
    #[vvm(port)]
    enable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, vvm::Sample)]
#[vvm(dut = crate::counter::Counter)]
pub struct CounterObservation {
    #[vvm(port)]
    count: u8,
}
```

## 8. Define A Clock

```rust
#[derive(Debug, Clone, Copy, Default, vvm::Clock)]
#[vvm(dut = crate::counter::Counter, clock = "clk")]
pub struct CounterClock;
```

## 9. Define A Sequence

The simplest sequence is just an iterator over typed stimulus values. The
maintained example uses a fixed sequence that covers reset, hold, count, and
reset again.

## 10. Add A Reference Model

The counter reference model tracks only the contract, not the RTL internals:

- reset drives the expected count to zero;
- enabled cycles increment it;
- disabled cycles hold it.

This is exactly the kind of small state machine a VVM `ReferenceModel` should
implement.

## 11. Add A Scoreboard

Use `ExactScoreboard` when the sampled observation must match the predicted one
exactly on every checked cycle.

## 12. Register A VVM Test

The maintained smoke test looks like this in structure:

```rust
#[vvm::test(trace, coverage)]
fn counter_smoke(context: &mut vvm::test::TestContext) -> Result<CounterTestResult> {
    let mut dut = Counter::new()?;

    context.config().configure_trace(&mut dut)?;

    Ok(vvm::testbench::Testbench::new(dut)
        .with_sequence(counter_sequence())
        .with_reference_model(CounterReferenceModel::default())
        .with_scoreboard(vvm::testbench::ExactScoreboard)
        .with_clock(CounterClock)
        .with_coverage(CounterCoverage::new("dut.counter")?)
        .run_covered::<CounterObservation>(context))
}
```

That one function ties together stimulus, observation, clocking, model,
scoreboard, tracing, and coverage.

## 13. Run The Test

```bash
cargo test counter_smoke
```

This remains an ordinary Rust test. The `#[vvm::test]` attribute generates the
Rust test registration, but Cargo still owns discovery and filtering.

## 14. Understand The Result

A passing run means:

- the DUT wrapper built successfully;
- the sequence executed under the configured clock;
- sampled observations matched the reference model;
- any attached coverage was captured.

If the test fails, the report includes cycle-aware diagnostics and, for replayable
tests, the token needed to reproduce the failure.

## 15. Enable Tracing

The smoke test already declares `trace`, so you only need to choose a directory:

```bash
VVM_TRACE_DIR=target/counter-traces cargo test counter_smoke
```

If `VVM_TRACE_DIR` is unset, VVM writes traces under `target/vvm-trace/`.

## 16. Replay A Randomized Failure

The maintained counter example includes a replayable randomized test named
`counter_random`. You can reproduce its exact pseudo-random sequence with:

```bash
VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test counter_random
```

`VVM_REPLAY` takes precedence over `VVM_SEED` for replay-capable tests.

## 17. What To Read Next

- Read [Verification Workflow](concepts/verification-workflow.md) for the model
  behind the quick start.
- Use [Creating A Testbench](guide/creating-a-testbench.md) when you want the
  runner phases in detail.
- Continue to the [Counter example chapter](examples/counter.md) for the full
  maintained example architecture.

## First-run Troubleshooting

- If `build.rs` cannot find Verilator, set `VERILATOR=/path/to/verilator`.
- If `include_dut!(counter)` fails, confirm the logical DUT name matches the one
  passed to `DutBuilder::new("counter")`.
- If generated accessors do not match your expectations, check whether your HDL
  port shape is covered by the current [Generated Types](reference/generated-types.md)
  support.
- If no trace appears, confirm the test declares `trace` and that the trace
  directory is writable.

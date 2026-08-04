# Quick Start

This quick start showcases the creation of a small VVM project from scratch.

Let's design and verify a tiny event counter that would work like this:

- has a `clk`, `reset_n` and `event_pulse` single bit logical inputs;
- a `total` output showing the counter total count;
- reset clears the counter total;
- each asserted event pulse increments the total on the next rising edge of
  `clk`;
- the output `total` shows the accumulated count.

## 1. Check Prerequisites

Confirm that Rust, Cargo, and Verilator are available:

```bash
rustc --version
cargo --version
verilator --version
```

If Verilator is not on `PATH`, see [Installation](installation.md).

## 2. Create The Project

Create a library crate so the verification code can live in normal Rust test
modules:

```bash
cargo new event-counter-verification --lib
```

Then add `build.rs` and your HDL sources under an `rtl/` directory so the project looks like this:

```text
event-counter-verification/
├── Cargo.toml
├── build.rs
├── rtl/
│   └── event_counter.sv
└── src/
    └── lib.rs
```

## 3. Understand The Project Files

- `Cargo.toml` declares the Rust package and its dependencies.
- `build.rs` runs before the crate compiles and asks VVM to generate the DUT
  wrapper.
- `rtl/event_counter.sv` contains the HDL design under test.
- `src/lib.rs` contains the generated-wrapper inclusion and the VVM test code.

## 4. Add Dependencies

Add VVM under test-only dependencies and `vvm-build` under build-time
dependencies:

```toml
{{#include ../../../tests/fixtures/docs-quick-start/Cargo.toml:dependencies}}
```

`vvm-rs` normally belongs under `[dev-dependencies]` because the generated DUT
wrapper and verification code are usually compiled only for tests. `vvm-build`
belongs under `[build-dependencies]` because Cargo runs it from `build.rs`.

## 5. Add RTL Design

Add HDL sources to `rtl/`:

`rtl/event_counter.sv`:
```systemverilog
{{#include ../../../tests/fixtures/docs-quick-start/rtl/event_counter.sv:rtl}}
```

The logical behaviour is simple on purpose: it lets you focus on the VVM
workflow instead of the design complexity.

## 6. Write `build.rs`

Add this build script:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/build.rs:build-script}}
```

### What This Means

- `DutBuilder::new("event_counter")` defines the logical DUT name for our VVM-generated DUT wrapper.
- `.top_module("event_counter")` selects the top HDL module.
- `.source("rtl/event_counter.sv")` points Verilator at the RTL source file.
- `.trace(...)` adds tracing capabilities to our DUT wrapper.
- `.build()` runs the Verilator-backed generation pipeline.

The string passed to `DutBuilder::new("event_counter")` defines the name of the generated DUT wrapper module and
must by matched by `vvm::include_dut!(...)` invocation when including the DUT in your verification code.

## 7. Include The Generated DUT

At the top of `src/lib.rs`, include the generated wrapper:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:include-dut}}
```

This macro loads the generated code from Cargo's `OUT_DIR`. You normally do not
edit that generated code directly.

## 8. Define A Stimulus Transaction

Add a struct that describes one semantic input step:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:stimulus}}
```

This transaction says what matters to the testbench:

- whether reset is asserted;
- whether an event pulse should be presented for that cycle.

Its goal is not to mirror any internal HDL detail. Its goal is to describe one
meaningful test input.

## 9. Define A Observation

Now define the output you want to sample after evaluation:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:observation}}
```

This keeps the sampled data focused on the user-visible result: in this case the total count
count.

## 10. Define A Clock

Add a clock type that tells VVM which DUT signal is the cycle clock:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:clock}}
```

## 11. Create A Deterministic Sequence

For a first test, use a simple, hand-written fixed sequence:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:sequence}}
```

This sequence covers:

- reset assertion;
- reset release;
- pulses that increment the count;
- a hold cycle with no event;
- a final reset.

## 12. Implement A Reference Model

The reference model predicts what the counter should do without depending on the
DUT implementation details:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:reference-model}}
```

This is the software-side behavioural contract of the designl, in this case:

- reset drives the expected total to zero;
- an event pulse increments it;
- otherwise the total holds.

## 13. Select A Scoreboard

This quick start uses `ExactScoreboard`, which means the observed and expected
values must match exactly on every checked cycle.

That is the right choice for a simple counter with one unambiguous expected
output.

## 14. Build A Testbench

The testbench is assembled inside a vvm test by chaining together the
DUT, sequence, reference model, scoreboard, and clock.

## 15. Register VVM Test

Add a complete VVM test:

```rust
{{#include ../../../tests/fixtures/docs-quick-start/src/lib.rs:test}}
```

This does several things at once:

- `#[vvm::test(trace)]` registers the test with Cargo and declares that tracing
  is supported.
- `EventCounter::new()?` constructs the generated DUT wrapper.
- `config.configure_trace(&mut dut)?` opens the trace if the run configuration
  requests one.
- `Testbench::new(dut)` starts the cycle-driven runner.
- `.run::<EventCounterObservation>()` drives, clocks, samples, predicts, and
  compares until the sequence is exhausted.

## 16. Run It

Run the single test with ordinary Cargo filtering:

```bash
cargo test event_counter_smoke
```

VVM integrates HDL simulation tests directly into native Rust test discovery.

## 17. Understand The Result

A passing run means:

- the Verilator-backed wrapper was generated successfully;
- the DUT constructed successfully;
- every sequence item was driven under the configured clock;
- every sampled observation matched the reference-model prediction.

If the run fails, VVM reports a structured mismatch or simulation error instead
of leaving you with only a raw boolean assertion.

## 18. Produce A Trace

Because the test declares `trace`, you can request a VCD file by setting a trace
directory:

```bash
VVM_TRACE_DIR=target/quick-start-traces cargo test event_counter_smoke
```

If `VVM_TRACE_DIR` is not set, VVM uses its default trace location under
`target/vvm-trace/`.

## 19. Understand What Cargo Generated

After the first build, Cargo and VVM have generated more than just your Rust test
binary.

At a high level, the build produced:

- Verilator-generated C++ model files;
- metadata describing the DUT ports;
- a generated Rust wrapper;
- compiled test artifacts under Cargo's target directory.

These generated files live under Cargo-managed output directories such as
`target/` and `OUT_DIR`. They are build artifacts, not source files to commit.

## 20. Common First-Run Problems

- If `build.rs` cannot find Verilator, set `VERILATOR=/path/to/verilator`.
- If `include_dut!(event_counter)` fails, confirm that the logical name matches
  `DutBuilder::new("event_counter")` exactly.
- If a derive fails, check that every `#[vvm(port)]` field matches a generated
  DUT port name and type.
- If no VCD file appears, confirm that the test declares `trace` and the trace
  directory is writable.

## 21. What To Read Next

- Read [How VVM Works](how-vvm-works.md) if you want the runtime model behind the
  code you just wrote.
- Read [Project Setup](guide/project-setup.md) and
  [Creating A Testbench](guide/creating-a-testbench.md) for the generalized
  workflow.
- Read [Configuring Tests](guide/configuring-tests.md) when you are ready to use
  environment-driven trace, replay, and cycle controls.

## 22. Related Case Study

Once you understand the pattern above, study the maintained
[Counter example](examples/counter.md) as a larger case study with additional
tests, coverage, and replay-driven workflows.

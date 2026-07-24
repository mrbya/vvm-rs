# VVM

Rust verification framework for Verilator-generated RTL models, including
internally scheduled HDL delays.

VVM provides strongly typed Rust testbenches, generated DUT bridges, and normal Cargo-based test execution for Verilated designs.

> [!WARNING]
> VVM is currently an early alpha. The core workflow is usable, but public APIs may change before the first stable release.

<!-- toc -->

- [What is VVM?](#what-is-vvm)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Bidirectional Ports](#bidirectional-ports)
- [Timing-enabled Models](#timing-enabled-models)
- [Functional Coverage](#functional-coverage)
- [Running VVM Tests](#running-vvm-tests)
- [Test Configuration](#test-configuration)
- [Workspace Crates](#workspace-crates)
- [Development](#development)
- [Documentation](#documentation)
- [Project Status](#project-status)
- [Similar Projects / Inspiration](#similar-projects--inspiration)
- [License](#license)

<!-- tocstop -->

## What is VVM?

VVM wraps Verilator-generated models in a Rust-first verification workflow.

At build time, `vvm-build` runs Verilator, generates the Rust/C++ bridge, and compiles the native support code needed for a DUT wrapper. At test time, `vvm` provides typed stimulus driving, output sampling, clock control, reference models, scoreboards, deterministic randomization, tracing, and a `#[vvm::test]` attribute that integrates directly with `cargo test` and `cargo nextest run`.

The working reference for public usage in this repository is `examples/counter`.

## Features

- Verilator model integration through `vvm-build` build-script APIs.
- Generated Rust/C++ DUT bridge code for supported Verilator designs.
- Strongly typed DUT driving and sampling via `Drive` and `Sample` derives.
- Typed clock abstractions via the `Clock` derive.
- Reusable typed testbench composition through `Testbench`.
- Ordinary iterator-based or replayable stimulus sequences.
- Stateful reference models and exact-equality scoreboards.
- Configurable failure policies with retained mismatch diagnostics.
- Explicit simulation time and cycle timing.
- Timing-enabled Verilator models through a typed build option.
- Explicit delayed-event processing through `TimedDut` and `TimingScheduler`.
- Manual delayed-slot stepping and bounded run-until-idle execution.
- Deterministic randomization with replay tokens.
- Structured pass/fail outcomes and detailed reports.
- VCD waveform tracing for trace-capable tests.
- `#[vvm::test]` for standard Rust test generation.
- Native `cargo test` and `cargo nextest run` execution.
- Standard Rust filtering, package selection, parallelism, and `#[ignore]` handling.
- Environment-based test configuration with replay, cycle, trace, and coverage overrides.
- Explicit Rust-native functional coverage with typed coverpoints and bins.

## Requirements

To build and run VVM-based tests for Verilated DUTs, the currently verified requirements are:

- Rust `1.87.0` or newer.
- A working C++ toolchain.
- Verilator.

Timing-enabled models require a C++ compiler with coroutine support. Ordinary
non-timing models retain the existing C++17 compilation path.

Additional contributor tooling used by this repository is installed by `just init`. That includes nightly Rust for formatting and dependency linting, `cargo-nextest`, `cargo-llvm-cov`, `cargo-udeps`, `cargo-audit`, `markdown-toc`, and `pre-commit`.

The CI Docker image also installs tools such as `clang`, `llvm`, `cmake`, `ninja`, `make`, `pkgconf`, and `npm`. Those are part of the repository's development and CI environment; they are not all required for ordinary VVM consumers.

## Installation

VVM is a library, not a command-line application. Add it to your project with Cargo dependencies instead of `cargo install`.

For a test-focused layout, use `vvm` as a development dependency and `vvm-build` in `build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }

[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

During pre-release evaluation, a temporary Git dependency also works:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", git = "https://gitlab.com/byacrates/vvm-rs.git" }

[build-dependencies]
vvm-build = { git = "https://gitlab.com/byacrates/vvm-rs.git" }
```

## Quick Start

Minimal project layout:

```text
counter-verification/
├── Cargo.toml
├── build.rs
├── rtl/
│   └── counter.sv
└── src/
    └── lib.rs
```

`build.rs`:

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

`src/lib.rs`:

```rust
#[cfg(test)]
mod vvm_tests {
    use vvm::prelude::*;

    vvm::include_dut!(counter);

    use crate::counter::Counter;

    #[derive(Clone, Copy, Debug, Drive)]
    #[vvm(dut = Counter)]
    struct CounterStimulus {
        #[vvm(port)]
        reset_n: bool,
        #[vvm(port)]
        enable: bool,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Sample)]
    #[vvm(dut = Counter)]
    struct CounterObservation {
        #[vvm(port)]
        count: u8,
    }

    #[derive(Clone, Copy, Debug, Default, Clock)]
    #[vvm(dut = Counter, clock = "clk")]
    struct CounterClock;

    type CounterTestResult = vvm::TestResult<
        CounterStimulus,
        vvm::Mismatch<CounterObservation, CounterObservation>,
        counter::CounterError,
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

Run the test with Cargo:

```console
cargo test counter_smoke
```

For unit-style VVM tests, keep them inside an explicit `#[cfg(test)]` module as shown above. Integration tests under `tests/` are already test-only and do not need an additional `#[cfg(test)]`.

## Bidirectional Ports

VVM supports plain packed-scalar top-level `inout` ports. Generated wrappers
separate the caller-presented input from the DUT's `output_enable` mask and
`output_value` proposal through `set_<port>_input`, `<port>_input()`,
`<port>_output_enable()`, `<port>_output_value()`, and `<port>()` returning
`InoutState`.

Resolution remains caller-owned: combine DUT and external enable/value
proposals, choose an explicit floating-bit policy, and write successful values
through `set_<port>_input`. Verilator execution is two-state, so Rust does not
receive `X` or `Z` values. See [`examples/tri-state-bus`](examples/tri-state-bus)
for bounded settling, exact contention detection, and VCD tracing.

## Timing-enabled Models

Timing mode builds a model that exposes internally scheduled HDL delays:

```rust
DutBuilder::new("delayed_sequence")
    .top_module("delayed_sequence")
    .source("rtl/delayed_sequence.sv")
    .timing()
    .build()
```

Run its finite delayed-event queue explicitly:

```rust
let mut scheduler = TimingScheduler::new();
let run = scheduler.run_until_idle(&mut dut, max_slots)?;
```

Cycle mode: Rust schedules external input-clock transitions and transaction
boundaries.

Timing mode: Verilator schedules internal delayed HDL processes.

The two schedulers are intentionally separate. See
[`examples/timing-delay`](examples/timing-delay) for the complete build,
stepping, tracing, and finalization example.

## Functional Coverage

VVM provides Rust-native functional coverage through typed coverpoints, bins,
and explicit two-way crosses. Normal, ignore, and illegal bins support exact
values, value sets, and inclusive ranges. Crosses consume successful
coverpoint samples, combine only normal-bin identities in deterministic
row-major order, and skip ignored or unmatched axes. Coverage remains an exact
integer ratio. Tests that capture coverage through a `TestContext` persist one
schema-v1 JSON artifact after the test runs; see
[`docs/coverage-json-v1.md`](docs/coverage-json-v1.md) for the file contract.

Coverage groups are ordinary user-defined structs. They retain typed sampling
while implementing `CoverageGroup` for read-only validation and aggregate
inspection of coverpoints and crosses.

Per-test artifacts can be merged explicitly after tests complete:

```rust
let merged = vvm::CoverageMerge::from_files(
    vvm::CoverageMergePolicy::passed_only(),
    artifact_paths,
)?;
merged.write_to("target/coverage/combined.vvmcov-merged.json")?;
```

Merging is offline, deterministic, and never updates shared test-runtime
state. See [`docs/coverage-merging.md`](docs/coverage-merging.md).

## Running VVM Tests

VVM tests are ordinary Rust tests.

Typical workflows:

```console
cargo test
cargo test counter
cargo test counter_random
cargo test --workspace
cargo nextest run
cargo nextest run -p <package>
```

Cargo and nextest own:

- test discovery;
- package and workspace selection;
- name filtering;
- parallel execution;
- ignored-test handling;
- standard summaries and reporting.

`cargo-nextest` is optional. If it is installed, VVM tests work with it directly.

## Test Configuration

VVM uses environment variables for global test configuration:

- `VVM_SEED`
- `VVM_REPLAY`
- `VVM_CYCLES`
- `VVM_TRACE_DIR`
- `VVM_COVERAGE_DIR`

Examples:

```console
VVM_SEED=0x1234 cargo test counter_random

VVM_CYCLES=100000 cargo test counter_random

VVM_REPLAY=chacha8-v1:0123456789abcdef \
    cargo test counter_random

VVM_TRACE_DIR=target/vvm-traces \
    cargo test counter_smoke

VVM_COVERAGE_DIR=target/vvm-coverage-artifacts \
    cargo test decoder_random
```

Replay precedence for replay-capable tests is:

1. `VVM_REPLAY`
2. `VVM_SEED`
3. Descriptor default replay token
4. Generated replay token

Capability behavior:

- replay overrides affect replay-capable tests;
- cycle overrides affect tests declaring cycle support;
- trace configuration affects tests declaring trace support.

If `VVM_TRACE_DIR` is not set, trace-capable tests write VCDs under a generated per-run directory rooted at `target/vvm-trace/`.

Tests that capture coverage write a separate `.vvmcov.json` artifact per test
under `VVM_COVERAGE_DIR`, or under a generated per-run directory rooted at
`target/vvm-coverage/` when it is unset. The bridge attempts this write after
the test body, including when the test has failed, and reports persistence
errors through the test result. No artifact is written when the test captures
no coverage.

## Workspace Crates

- `vvm-rs` / `vvm`: public facade crate used by verification code.
- `vvm-core`: runtime verification primitives, outcomes, timing, replay, and testbench execution.
- `vvm-build`: Verilator invocation, metadata handling, code generation, and native bridge compilation.
- `vvm-macros`: derives for drive/sample/clock plus `#[vvm::test]`.
- `vvm-example-counter`: minimal cycle-driven verification.
- `vvm-example-multi-clock`: independently timed externally driven clocks.
- `vvm-example-timing-delay`: internally scheduled HDL delays.
- `vvm-example-tri-state-bus`: caller-owned top-level inout resolution.

## Development

The repository uses `just` recipes as the primary contributor interface.

First-time setup:

```console
cargo install just
just init
```

Common commands:

```console
just fmt --check
just check -- -D warnings
just test
just doctest
just ci
```

`just ci` is the repository's CI-equivalent verification command.

## Documentation

Generate local API docs with:

```console
cargo doc --workspace --no-deps --open
```

## Project Status

`0.1.0-alpha.1` is the first public alpha release for this workspace.

- The current core workflow is implemented and verified in CI.
- Public APIs may still change during alpha.
- Development continues beyond the initial MVP.
- Practical feedback and real verification examples are especially useful at this stage.

## Similar Projects / Inspiration

VVM is heavily inspired by [UVM](https://www.accellera.org/downloads/standards/uvm) and is a reboot of the original [VVM](docs/dev/reference-projects/vvm) reference project preserved in this repository.

## License

Licensed under either of the following, at your option:

- Apache License 2.0 (`LICENSE-APACHE`)
- MIT (`LICENSE-MIT`)

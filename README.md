# VVM

Rust-first verification for Verilator-generated HDL models.

VVM gives a Rust crate a generated DUT wrapper, typed drive/sample APIs,
testbench composition, replayable randomization, waveform tracing, timing-mode
support for delayed HDL events, and Rust-native functional coverage.

> [!IMPORTANT]
> VVM is a pre-`1.0` verification framework. The public API planned for `v0.2.0`
> is established for this release cycle, milestone 12.8 is the final audit
> before release-candidate freeze, milestone 12.9 will handle
> `v0.2.0-rc.1` publication and dogfooding, supported workflows are CI-tested,
> and documented platform and HDL limitations still apply.

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

For a fuller comparison against commercial simulators, vendor tools, cocotb, and
raw Verilator+C++, read the following book chapter:
<https://byacrates.gitlab.io/vvm-rs/why-vvm.html>

## Current Status

- Linux-only native verification support for the `v0.2.0` release line.
- CI validates Rust 1.87.0, the current stable Rust toolchain, Verilator 5.000,
  Verilator 5.050, and GCC-based native builds.
- Native fixture coverage also validates a Linux Clang toolchain.
- Two-state Verilator behavior: Rust-visible ports do not carry HDL `X` or `Z`.
- Timing mode supports delayed future slots, not same-time or `#0` scheduling.
- Pre-`1.0` semantic versioning: patch releases in the `0.2.x` line preserve the
  supported public API, while a future pre-`1.0` minor release may make
  intentional breaking changes.

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

- Rust 1.87.0 or the current stable Rust toolchain.
- Verilator.
- A working Linux C++ toolchain.

Ordinary native models use a C++17-capable toolchain.

Timing-enabled models additionally need a C++20 compiler with coroutine support.

## Installation

Most users add the facade crate as `vvm` and use `vvm-build` in `build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.2.0" }

[build-dependencies]
vvm-build = "0.2.0"
```

Use a Git dependency from the default branch when you need unreleased
development changes before `v0.2.0` is published.

Install `cargo-vvm` when you want suite-level functional-coverage merge and
reporting:

```bash
cargo install cargo-vvm
```

## Crate layout

- `vvm-build` runs Verilator during `build.rs`, generates rust-to-c++ bridges, and compiles rust-native wrappers.
- `vvm` is the public VVM library entrypoint.
- `cargo-vvm` is a cargo utility cli tool to manage functional coverage artifacts and reports.

## Quickstart

The full from-scratch walkthrough lives in the book:

- <https://byacrates.gitlab.io/vvm-rs/quick-start.html>

The minimal project shape is:

```text
counter-verification/
├── Cargo.toml
├── build.rs        # vvm-build Verilator bootstrap
├── rtl/
│   └── counter.sv  # your design HDL source
└── src/
    └── lib.rs      # vvm rust testbench and test setup
```

The common build-time step is a small `build.rs` such as:

```rust
use vvm_build::{BuildResult, DutBuilder, TraceOptions};

fn main() -> BuildResult<()> {
    DutBuilder::new("event_counter")
        .top_module("event_counter")
        .source("rtl/event_counter.sv")
        .trace(TraceOptions::vcd())
        .build()
}
```

The complete HDL, generated-wrapper inclusion, typed stimulus and observation,
reference model, scoreboard, and registered test are in the Quick Start chapter
instead of being partially duplicated here.

Run tests with normal Cargo commands:

```bash
cargo test
# or
cargo nextest run
```

## Example Ladder

- `examples/counter`: first larger case study after the Quick Start.
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

Local Criterion benchmarks:

```bash
just benchmark
just benchmark-save-baseline before-change
just benchmark-compare-baseline before-change
just benchmark-target vvm-core packed
```

These benchmarks are local-only and machine-specific. See the book's
development benchmarking chapter for the full workflow and policy.

## Documentation

- Project book: <https://byacrates.gitlab.io/vvm-rs/>
- Quick start: <https://byacrates.gitlab.io/vvm-rs/quick-start.html>
- API reference: <https://byacrates.gitlab.io/vvm-rs/api/>
- Example ladder: <https://byacrates.gitlab.io/vvm-rs/examples.html>

## Similar Projects And Inspiration

VVM is heavily inspired by UVM-style verification structure and by the original
VVM reference project.

## License

Licensed under either of the following, at your option:

- Apache License 2.0 (`LICENSE-APACHE`)
- MIT (`LICENSE-MIT`)

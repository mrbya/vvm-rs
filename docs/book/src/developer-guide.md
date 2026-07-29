# Developer guide

The workspace has four library boundaries: `vvm-rs` is the facade, `vvm-build` owns build-script Verilator invocation and code generation, `vvm-core` owns pure-Rust runtime primitives, and `vvm-macros` owns derives and `#[vvm::test]`. `cargo-vvm` is a direct-entry binary package; examples and fixtures are consumers rather than public API crates.

Build-time flow is source configuration, Verilator metadata generation, normalization, HDL type mapping, C++ adapter generation, CXX bridge generation, Rust wrapper generation, native compilation, then consumer inclusion from `OUT_DIR`. Generated adapter code owns the Verilated context and model; generated Rust hides CXX pointers and pins. The direct model-member ABI is confined to the private C++ implementation.

Runtime flow is initialization, drive, evaluation, clock transition, sampling, model prediction, scoreboard check, coverage observation, finalization, and trace closure. `ClockScheduler` and `TimingScheduler` remain separate. Randomization owns explicit replay tokens. Coverage snapshots are immutable and merge offline. Macro expansion preserves normal Rust discovery and produces compile-time diagnostics for invalid derives.

Unsafe and FFI invariants are narrow: C++ exceptions never cross CXX, Rust users do not receive raw native ownership, generated ABI symbols remain internal, and a DUT is not assumed thread-safe. Errors retain their stage so build, simulation, coverage, and reporting faults remain distinguishable.

Maintainers should use [Contributing](contributing.md), the [testing strategy](../../dev/testing-strategy.md), and the [example strategy](../../dev/example-strategy.md). Stable public contracts belong in rustdoc; this guide describes architecture and implementation boundaries.

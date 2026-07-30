# Introduction

VVM is a Rust-first verification framework for Verilator-generated HDL models.
It gives a Rust project a typed generated DUT wrapper, a testbench runner,
deterministic replay, waveform tracing, timing support for delayed HDL events,
and functional coverage that stays in normal Cargo workflows.

VVM is for engineers who want to write verification logic in Rust instead of in
an HDL-only testbench or a full UVM environment. The normal VVM workflow is:

1. Add `vvm-build` in `build.rs` to run Verilator and generate the wrapper.
2. Include that wrapper with `vvm::include_dut!`.
3. Define typed stimulus, observation, and clock types.
4. Run sequences through a `Testbench` with a reference model, scoreboard, and
   optional coverage.
5. Execute the resulting test through `cargo test` or `cargo nextest run`.

VVM is not a four-state simulator, a UVM compatibility layer, or a replacement
for timing-accurate CDC analysis. Verilator remains the HDL execution engine,
so the integration is fundamentally two-state and Linux-focused today.

## Components

- `vvm-rs` / `vvm`: public facade for test authors.
- `vvm-build`: build-script crate that runs Verilator and generates the bridge.
- `cargo-vvm`: CLI for offline functional-coverage merge and reporting.
- Verilator: HDL compiler and simulation engine used under the hood.
- Generated DUT wrapper: typed Rust API compiled into the consuming crate.

## Current Status

VVM is an early alpha. The core workflow is real and tested, but public APIs may
still move before a stable release.

Current support notes:

- Rust edition 2024 with MSRV 1.87.0.
- Native verification is Linux-focused.
- Ordinary models use the existing C++17 path.
- Timing-enabled models additionally require coroutine-capable C++ support.
- CI verifies Verilator 5.000 as the minimum supported version and 5.050 as the
  current tested version.

## Important Limitations

- Verilator is two-state in this integration. Rust-visible ports do not carry
  HDL `X` or `Z` values.
- Timing mode supports delayed future slots, but not same-time or `#0`
  scheduling.
- Multi-clock examples demonstrate deterministic execution ordering, not formal
  CDC proof or metastability modeling.
- Unsupported or awkward HDL port shapes may still require fixture-driven
  validation rather than polished user-facing workflows.

## Where To Go Next

- Read [Quick Start](quick-start.md) for a complete first test.
- Use [Installation](installation.md) for prerequisites and contributor setup.
- Read [Concepts](concepts/overview.md) before diving into API details.
- Use the [API Guide](api-guide/overview.md) when you need the public facade in
  workflow terms.
- Follow the [Examples](examples.md) ladder when you want a tested reference
  design to study.

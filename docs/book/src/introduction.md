# Introduction

VVM is a Rust-first verification framework for Verilator-generated HDL models.

It is aimed primarily at RTL and HDL engineers who want to keep a compiled,
open-source verification flow while writing the testbench side in Rust instead of
hand-owning a C++ harnesses for Verilator or staying inside an HDL-only environment.

VVM provides:

- generated typed DUT wrappers;
- typed drive, sample, and clock derives;
- testbench orchestration with sequences, reference models, and scoreboards;
- deterministic replay;
- waveform tracing;
- timing and multi-clock support;
- functional coverage that stays inside ordinary Cargo workflows.

VVM is in pre-`1.0` development. The public API intended for `v0.2.0` is
established for this release cycle, supported workflows are tested, and the
documented support surface remains intentionally explicit.

VVM is not a four-state simulator, not a UVM replacement, and not a substitute
for dedicated CDC or metastability tools. Verilator remains the actual HDL
execution engine, so the common integration path is two-state and Linux-focused
today.

## Components

- `vvm-rs` / `vvm`: public facade of VVM.
- `vvm-build`: build-script crate that runs Verilator and generates vvm rust wrappers and cxx bridge to verilated designs.
- `cargo-vvm`: CLI for offline functional-coverage artifact and reporting management.
- Verilator: HDL compiler and simulation engine used under the hood.
- Generated DUT wrapper: typed Rust API compiled into the consuming crate.

## Current Status

VVM is no longer presented as an alpha prototype. It remains pre-`1.0`, so a
future minor release may still make intentional breaking changes, but the
`v0.2.0` release cycle treats the reviewed public API as frozen.

Current support notes:

- Rust edition 2024 with MSRV 1.87.0.
- Native verification is Linux-focused.
- Ordinary models use the existing C++17 path.
- Timing-enabled models additionally require coroutine-capable C++ support.
- Verilator 5.000 as the minimum supported version and 5.050 as the current
  validated version.

## Important Limitations

- Verilator is two-state in this integration. Rust-visible ports do not carry
  HDL `X` or `Z` values.
- Timing mode supports delayed future slots, but not same-time or `#0`
  scheduling.
- Multi-clock api manifests deterministic execution ordering, not formal
  CDC proof or metastability modeling.
- Unsupported or awkward HDL port shapes may still require fixture-driven
  validation rather than polished user-facing workflows.

## Where To Go Next

- Read [Why VVM?](why-vvm.md) for the problem statement and tool-positioning.
- Read [How VVM Fits Into HDL Verification](how-vvm-fits-into-hdl-verification.md) for the build and test pipeline.
- Read [How VVM Works](how-vvm-works.md) for the runtime mental model.
- Use [Installation](installation.md) and [Quick Start](quick-start.md) when you
  are ready to build a project.
- Read [Verification Workflow](concepts/verification-workflow.md) for the mental model and [Project Setup](guide/project-setup.md) to continue through the full Guide.

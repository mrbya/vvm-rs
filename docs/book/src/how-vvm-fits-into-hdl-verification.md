# How VVM Fits Into HDL Verification

VVM is not the simulation engine. Verilator is.

VVM sits above Verilator and turns the usual generated-model integration work
into a Rust and Cargo workflow.

## Build And Test Pipeline

```text
SystemVerilog / Verilog sources
             |
             v
          build.rs
        + vvm-build
             |
             v
          Verilator
             |
             v
Generated C++ model and metadata
             |
             v
Generated CXX bridge and Rust DUT wrapper
             |
             v
Rust verification code using vvm
             |
             v
cargo test / cargo nextest
             |
             |-- diagnostics
             |-- replay information
             |-- VCD traces
             '-- coverage artifacts
```

## What Runs At Cargo Build Time

When Cargo sees `build = "build.rs"` in your package, it runs `build.rs` before
compiling your crate.

In a VVM project, `build.rs` typically:

1. creates a `DutBuilder`;
3. lists HDL sources and build options;
3. asks `vvm-build` to invoke Verilator to verilate your design and generate the Rust-facing wrapper and bridge.

That generated code is written under Cargo's `OUT_DIR` among other build artifacts. You normally do not edit or commit it.

## What Verilator Owns

Verilator owns HDL elaboration and compiled simulation of the generated C++
model.

The model's behaviour, supported language subset, timing capabilities, and
two-state semantics all come from that Verilator-based execution path.

## What VVM Owns

VVM owns the Rust-side integration layer:

- generated typed DUT wrappers;
- typed drive and sample derives;
- testbench orchestration;
- test registration;
- replay handling;
- trace configuration;
- timing and multi-clock orchestration helpers;
- coverage capture and reporting hooks.

## What You Write

You normally write:

- HDL source files;
- a small declarative `build.rs`;
- Rust stimulus, observation, clock, sequence, reference-model, scoreboard, and
  test code.

## Cargo as the Top-Level Workflow

VVM is designed so the top-level command remains a normal Rust test command such
as:

```bash
cargo test
```

That keeps build, test selection, result reporting, and CI integration aligned
with ordinary Cargo workflows instead of requiring a separate custom runner for
the common case.

Read [How VVM Works](how-vvm-works.md) next for the runtime cycle inside one VVM
test.

# VVM-rs Implementation Plan

> **Project:** VVM-rs — Verilator Verification Methodology in Rust
> **Status:** Pre-release stabilization toward VVM v0.2.0
> **Primary goal:** Build ergonomic, strongly typed Rust testbenches for Verilator-generated HDL models.

This document is the working roadmap for VVM-rs. Check tasks off as they are completed and update the milestone table after each meaningful change.

---

## 1. Project vision

VVM-rs should allow a user to:

1. Describe one or more HDL sources in `build.rs`.
2. Invoke Verilator automatically.
3. Generate a strongly typed C++ adapter and CXX bridge.
4. Expose a safe Rust wrapper for the generated DUT.
5. Define stimuli and observations as ordinary Rust types.
6. Derive repetitive `Drive` and `Sample` implementations.
7. Run deterministic testbenches with reference models and scoreboards.
8. Produce useful diagnostics and waveforms.

The intended end-user workflow should eventually resemble:

```rust
// build.rs

fn main() -> Result<(), vvm_build::BuildError> {
    vvm_build::DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .build()
}
```

```rust
use vvm::{Drive, Sample, Testbench};

vvm::include_dut!(counter);

#[derive(Clone, Debug, Drive)]
#[vvm(dut = Counter)]
struct Stimulus {
    #[vvm(port)]
    reset_n: bool,

    #[vvm(port)]
    enable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Sample)]
#[vvm(dut = Counter)]
struct Observation {
    #[vvm(port)]
    count: u8,
}

fn main() -> vvm::Result<()> {
    Testbench::new(Counter::new()?)
        .clock(|dut, level| dut.set_clk(level))
        .sequence(sequence())
        .reference_model(CounterModel::default())
        .scoreboard(vvm::ExactScoreboard::default())
        .run()?
        .report()
}
```

This is the target API, not the starting point.

---

## 2. Guiding architectural rules

- [ ] Build vertical slices rather than completing isolated crates.
- [ ] End every major milestone with a runnable HDL example.
- [ ] Handwrite an integration before generating it.
- [ ] Implement a trait manually before introducing a derive macro for it.
- [ ] Keep `vvm-core` independent of Verilator, CXX, and generated DUT types.
- [ ] Keep native and unsafe details out of normal user testbench code.
- [ ] Reject unsupported HDL constructs explicitly during the build.
- [ ] Do not reproduce UVM concepts unless they improve the Rust API.
- [ ] Avoid asynchronous runtimes until a concrete requirement justifies one.
- [ ] Prefer standard Rust traits such as `Iterator` before defining equivalents.
- [ ] Keep generated code mechanically simple and inspectable.
- [ ] Treat the first working example as the source of truth for API design.
- [ ] Keep the public facade minimal until lower-level APIs have stabilized.
- [ ] Preserve deterministic simulation ordering and document it explicitly.
- [ ] Add abstractions only after identifying repeated working code.

---

## 3. Crate responsibilities

### `vvm-build`

Used from a consuming crate's `build.rs`.

Responsibilities:

- Locate and invoke Verilator.
- Detect or validate the Verilator version.
- Manage generated output directories.
- Emit Cargo rebuild and linking directives.
- Extract and normalize Verilator metadata.
- Generate the C++ adapter.
- Generate the CXX bridge.
- Generate the safe Rust DUT wrapper.
- Compile and link native code.
- Report useful structured build errors.

Must not contain simulation methodology, scoreboards, sequences, or reference models.

### `vvm-core`

Pure Rust verification functionality:

- `Dut`
- `Drive`
- `Sample`
- `ReferenceModel`
- `Scoreboard`
- Testbench execution
- Clock and reset helpers
- Test results and mismatch diagnostics
- Deterministic cycle scheduling

It must not depend on Verilator, CXX, `vvm-build`, generated DUT classes, or procedural macro internals.

### `vvm-macros`

Procedural macros only:

- `#[derive(Drive)]`
- `#[derive(Sample)]`
- `#[vvm(...)]` helper attribute parsing

Possible later additions include test registration and field-comparison helpers.

### `vvm`

Public facade.

Initial state:

- Crate documentation only.
- No placeholder API.
- No premature re-exports.

Later responsibilities:

- Re-export stable `vvm-core` traits and types.
- Re-export stable procedural macros.
- Provide a small prelude.
- Provide convenience macros such as `include_dut!`.

### `vvm-ffi`

Removed. Shared ABI support should only return once concrete cross-DUT native functionality exists.

---

## 4. Milestone status

| Milestone | Name | Status |
|---:|---|---|
| 0 | Workspace preparation | Complete |
| 1 | Handwritten Rust ↔ CXX ↔ Verilator bridge | Complete |
| 2 | Reusable Verilator build orchestration | Complete |
| 3 | Verilator metadata extraction | Complete |
| 4 | Generated DUT bridge | Complete |
| 5 | Safe generated DUT API | Complete |
| 6 | Minimal pure-Rust VVM core | Complete |
| 7 | Testbench runner | Complete |
| 8 | Derive macros | Complete |
| 9 | Public facade | Complete |
| 10 | Tracing, reporting, and standard test harness | Complete |
| 11 | Wider HDL feature support and Rust-native coverage | Complete |
| 12 | Pre-release cleanup and polish | In progress (12.5 documentation and Pages complete; 12.6 benchmark suite and local baselines complete; 12.7 packaging not started) |

---

# 5. Milestone 0 — Workspace preparation

## Goal

Prepare the workspace for the first real CXX/Verilator integration without designing the VVM runtime prematurely.

## Existing completed bootstrap work

- [x] Use a virtual Cargo workspace.
- [x] Remove the workspace root itself from `workspace.members`.
- [x] Create the `vvm` facade crate.
- [x] Create the `vvm-build` crate.
- [x] Create the `vvm-core` crate.
- [x] Create the `vvm-macros` crate.
- [x] Configure `vvm-macros` as a procedural macro crate.
- [x] Add workspace-level `proc-macro2`.
- [x] Add workspace-level `quote`.
- [x] Add workspace-level `syn`.

## Workspace cleanup

- [x] Remove placeholder `add()` functions from all crates.
- [x] Remove placeholder tests created by the bootstrap template.
- [x] Verify every crate has accurate crate-level documentation.
- [x] Correct typos in crate documentation and package descriptions.
- [x] Review package names versus Rust import names.
- [x] Decide whether the facade package remains `vvm-rs` with library name `vvm`.
- [x] Review homepage, repository, keywords, and categories.
- [x] Confirm the minimum supported Rust version.
- [x] Decide whether repeated lint declarations should move to workspace lints.
- [x] Ensure empty crates compile without artificial APIs.

## Dependencies

Add dependencies only when needed.

- [x] Add workspace dependency `cxx`.
- [x] Add workspace dependency `cxx-build`.
- [x] Add workspace dependency `thiserror`.
- [x] Add workspace dev-dependency `trybuild`.
- [x] Enable required `syn` features only when macro implementation begins.
- [x] Avoid serialization dependencies until metadata parsing begins.

## Counter example package

Create:

```text
examples/counter/
├── Cargo.toml
├── build.rs
├── rtl/
│   └── counter.sv
├── cpp/
│   ├── counter.cpp
│   └── counter.hpp
└── src/
    ├── bridge.rs
    └── main.rs
```

- [x] Add `examples/counter` as a workspace member.
- [x] Create the example package manifest.
- [x] Add `cxx` as a normal dependency.
- [x] Add `cxx-build` as a build dependency.
- [x] Add the first counter RTL module.
- [x] Add an initially minimal `build.rs`.
- [x] Add an initially minimal Rust executable.

Suggested DUT:

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

- [x] Verify the module parses with Verilator independently.
- [x] Decide whether HDL lint warnings are fatal in examples.
- [x] Document expected reset and counting behavior.

## Acceptance criteria

- [x] `cargo check --workspace` passes.
- [x] `cargo test --workspace` passes.
- [x] `cargo clippy --workspace --all-targets` passes.
- [x] The counter HDL can be linted by Verilator.
- [x] No placeholder public APIs remain.

---

# 6. Milestone 1 — Handwritten Rust ↔ CXX ↔ Verilator bridge

## Goal

Prove that Rust can construct, drive, evaluate, observe, and destroy a Verilated DUT through a handwritten CXX bridge.

No VVM methodology abstractions and no VVM-generated bridge code are allowed in this milestone.

## Data flow

```text
Rust executable
    │
    ▼
handwritten #[cxx::bridge]
    │
    ▼
handwritten C++ adapter
    │
    ▼
Verilator-generated Vcounter
```

## Verilator invocation

- [x] Read `OUT_DIR`.
- [x] Create a deterministic Verilator output directory.
- [x] Emit `cargo::rerun-if-changed=rtl/counter.sv`.
- [x] Locate the `verilator` executable.
- [x] Invoke Verilator with `--cc`.
- [x] Set `--top-module counter`.
- [x] Set `--prefix Vcounter`.
- [x] Set `--Mdir` below `OUT_DIR`.
- [x] Enable `--emit-accessors`.
- [x] Build the generated model.
- [x] Capture and display useful command failures.
- [x] Avoid shell command strings; use `std::process::Command`.

## Handwritten C++ adapter

Suggested public header:

```cpp
#pragma once

#include <cstdint>
#include <memory>

namespace vvm::counter {

class Counter final {
public:
    static std::unique_ptr<Counter> create();

    ~Counter();

    void eval() noexcept;
    void finish() noexcept;

    void set_clk(bool value) noexcept;
    void set_reset_n(bool value) noexcept;
    void set_enable(bool value) noexcept;

    [[nodiscard]] std::uint8_t count() const noexcept;

private:
    Counter();

    class Impl;
    std::unique_ptr<Impl> impl_;
};

}
```

- [x] Implement the adapter with PIMPL.
- [x] Own a `VerilatedContext`.
- [x] Own a `Vcounter`.
- [x] Implement construction without leaking exceptions across FFI.
- [x] Implement `eval()`.
- [x] Implement `finish()`.
- [x] Ensure `final()` is called at most once.
- [x] Call `finish()` from the destructor if needed.
- [x] Implement typed port accessors.
- [x] Keep Verilator headers out of the CXX-visible public header.

## Handwritten CXX bridge

```rust
#[cxx::bridge(namespace = "vvm::counter")]
mod ffi {
    unsafe extern "C++" {
        include!("counter/counter.hpp");

        type Counter;

        fn create() -> UniquePtr<Counter>;

        fn eval(self: Pin<&mut Counter>);
        fn finish(self: Pin<&mut Counter>);

        fn set_clk(self: Pin<&mut Counter>, value: bool);
        fn set_reset_n(self: Pin<&mut Counter>, value: bool);
        fn set_enable(self: Pin<&mut Counter>, value: bool);

        fn count(self: &Counter) -> u8;
    }
}
```

- [x] Create the bridge module.
- [x] Configure `cxx_build::bridge`.
- [x] Compile the handwritten C++ adapter.
- [x] Add required include directories.
- [x] Compile the Verilated model sources into the example binary.
- [x] Link the C++ standard library as needed.
- [x] Verify bridge signature checks pass.
- [x] Keep the raw FFI module private.

## Safe Rust wrapper

- [x] Create a safe `Counter` wrapper.
- [x] Store `cxx::UniquePtr<ffi::Counter>` privately.
- [x] Reject null construction.
- [x] Expose safe `eval()`.
- [x] Expose safe setters.
- [x] Expose safe output getters.
- [x] Expose explicit `finish()`.
- [x] Ensure dropping the wrapper is safe.
- [x] Do not expose `Pin`, `UniquePtr`, or raw FFI types to `main.rs`.

## Manual simulation

Initial scheduling sequence:

1. Set clock low.
2. Assert reset.
3. Evaluate.
4. Raise clock.
5. Evaluate.
6. Confirm count is zero.
7. Lower clock.
8. Release reset.
9. Enable counting.
10. Evaluate.
11. Raise clock.
12. Evaluate.
13. Confirm count incremented.

- [x] Implement reset behavior check.
- [x] Implement at least three count cycles.
- [x] Verify disabled cycles do not increment.
- [x] Print cycle and count values.
- [x] Return a non-zero process result on mismatch.
- [x] Confirm no Verilator object leaks.
- [x] Confirm `final()` executes.

## Acceptance criteria

- [x] `cargo test -p vvm-example-counter counter_smoke` succeeds.
- [x] Count remains zero during reset.
- [x] Count increments on rising edges while enabled.
- [x] Count does not increment while disabled.
- [x] CXX details are hidden behind a safe local Rust wrapper.
- [x] No VVM bridge generation exists yet.

---

# 7. Milestone 2 — Reusable Verilator build orchestration

## Goal

Move build mechanics from the counter example into `vvm-build`, while retaining the handwritten adapter and bridge.

Target `build.rs`:

```rust
fn main() -> Result<(), vvm_build::BuildError> {
    vvm_build::DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .bridge("src/bridge.rs")
        .cpp_source("cpp/counter.cpp")
        .include("cpp")
        .build()
}
```

## Suggested module structure

```text
vvm-build/src/
├── lib.rs
├── builder.rs
├── cargo.rs
├── command.rs
├── error.rs
├── paths.rs
└── verilator.rs
```

## Builder configuration

- [ ] Add `DutBuilder`.
- [ ] Store a logical DUT name.
- [ ] Store the top-module name.
- [ ] Support multiple HDL source files.
- [ ] Support HDL include directories.
- [ ] Support HDL defines.
- [ ] Support additional Verilator arguments.
- [ ] Store the handwritten CXX bridge path.
- [ ] Support one or more handwritten C++ sources.
- [ ] Support C++ include directories.
- [ ] Validate required fields before invoking tools.

## Command execution

- [ ] Add a reusable checked-command runner.
- [ ] Capture stdout and stderr.
- [ ] Capture the exit status.
- [ ] Include the rendered command in failures.
- [ ] Preserve non-UTF-8 paths where possible.
- [ ] Avoid `unwrap()` and `expect()`.
- [ ] Avoid panic-prone indexing.
- [ ] Unit-test command rendering where practical.

## Path management

- [ ] Read `OUT_DIR`.
- [ ] Create a stable per-DUT directory.
- [ ] Separate Verilator, generated, and metadata outputs.
- [ ] Avoid deleting unrelated outputs.
- [ ] Normalize relative paths against `CARGO_MANIFEST_DIR`.
- [ ] Detect duplicate source entries.
- [ ] Return contextual I/O errors.

## Cargo integration

- [ ] Emit `rerun-if-changed` for every HDL source.
- [ ] Emit `rerun-if-changed` for bridge and C++ files.
- [ ] Emit `rerun-if-env-changed=VERILATOR`.
- [ ] Emit `rerun-if-env-changed=VERILATOR_ROOT`.
- [ ] Emit native link-search directives.
- [ ] Emit native link-library directives.
- [ ] Avoid unnecessary rebuilds.

## Verilator discovery

- [ ] Respect a user-provided `VERILATOR` executable path.
- [ ] Fall back to `PATH`.
- [ ] Query the Verilator version.
- [ ] Obtain `VERILATOR_ROOT`.
- [ ] Validate required runtime files.
- [ ] Produce a clear error when Verilator is absent.
- [ ] Record supported version assumptions.

## Acceptance criteria

- [ ] Counter `build.rs` contains configuration only.
- [ ] Counter behavior is unchanged from Milestone 1.
- [ ] Build failures include the failed command and stderr.
- [ ] Unrelated file changes do not rebuild the model.
- [ ] `vvm-build` has tests for validation and path handling.

---

# 8. Milestone 3 — Verilator metadata extraction

## Goal

Discover and normalize DUT top-level ports without parsing generated C++ headers.

## Metadata invocation

- [ ] Add a separate Verilator metadata command.
- [ ] Invoke Verilator with `--json-only`.
- [ ] Write metadata below the DUT output directory.
- [ ] Keep metadata extraction separate from model compilation.
- [ ] Reuse top module, sources, includes, and defines.
- [ ] Ensure argument ordering is deterministic.
- [ ] Record the Verilator version with generated metadata.

## Normalized metadata

```rust
pub struct DutMetadata {
    pub name: String,
    pub top_module: String,
    pub ports: Vec<Port>,
}

pub struct Port {
    pub name: String,
    pub direction: PortDirection,
    pub width: BitWidth,
    pub signed: bool,
}

pub enum PortDirection {
    Input,
    Output,
    Inout,
}
```

- [x] Define normalized metadata types.
- [x] Keep raw JSON structures private.
- [x] Preserve HDL names and port order.
- [x] Validate unique port names.
- [x] Represent widths with a non-zero type.
- [x] Record signedness.
- [x] Record dimensions and aggregate kinds.
- [x] Separate parse errors from unsupported-feature errors.

## Initially supported

- [x] One-bit inputs.
- [x] One-bit outputs.
- [x] Unsigned packed inputs from 2 to 64 bits.
- [x] Unsigned packed outputs from 2 to 64 bits.

## Initially rejected

- [x] Inout ports.
- [x] Signed ports.
- [x] Ports wider than 64 bits.
- [x] Unpacked arrays.
- [x] Packed structs and unions.
- [x] Interfaces.
- [x] Real-valued ports.
- [x] Unsupported four-state representations.

Every rejection should name the signal and explain why it is unsupported.

## Parser fixtures

- [x] Store representative Verilator JSON fixtures.
- [x] Record the generating Verilator version.
- [x] Test parsing without requiring Verilator.
- [x] Test malformed JSON.
- [x] Test missing top-module metadata.
- [x] Test unsupported ports.
- [x] Test empty modules.
- [x] Test deterministic normalized output.

## Acceptance criteria

- [x] Counter metadata contains `clk`, `reset_n`, `enable`, and `count`.
- [x] Directions and widths are correct.
- [x] Parser tests run without Verilator.
- [x] Unsupported constructs fail with precise diagnostics.
- [x] The handwritten bridge remains in use for this milestone.

---

# 9. Milestone 4 — Generated DUT bridge

## Goal

Generate the C++ adapter, CXX bridge, and safe Rust wrapper from normalized metadata.

## Generated layout

```text
OUT_DIR/vvm/counter/
├── verilated/
│   └── ...
├── generated/
│   ├── counter.hpp
│   ├── counter.cpp
│   ├── bridge.rs
│   └── dut.rs
└── metadata/
    ├── raw.json
    └── normalized.json
```

## Code generation structure

```text
vvm-build/src/codegen/
├── mod.rs
├── cpp_adapter.rs
├── cxx_bridge.rs
├── rust_wrapper.rs
└── names.rs
```

## Naming rules

- [x] Convert HDL module names to safe Rust type names.
- [x] Convert HDL port names to safe Rust method names.
- [x] Preserve original names in metadata.
- [x] Handle Rust and C++ keywords.
- [x] Reject irreconcilable naming collisions.
- [x] Use deterministic generated symbols.
- [x] Avoid exposing Verilator naming conventions.

## Initial signal mapping

| HDL width | C++ type | Rust type |
|---:|---|---|
| 1 | `bool` | `bool` |
| 2–8 | `std::uint8_t` | `u8` |
| 9–16 | `std::uint16_t` | `u16` |
| 17–32 | `std::uint32_t` | `u32` |
| 33–64 | `std::uint64_t` | `u64` |

- [x] Implement width-to-C++ mapping.
- [x] Implement width-to-Rust mapping.
- [x] Decide whether narrow input values are masked or rejected.
- [x] Document the chosen behavior.
- [x] Unit-test all boundary widths.

## Generated C++ adapter

- [x] Generate a PIMPL-based header.
- [x] Generate model/context ownership.
- [x] Generate construction.
- [x] Generate `eval()` and `finish()`.
- [x] Generate destructor finalization.
- [x] Generate setters only for inputs.
- [x] Generate getters only for outputs.
- [x] Use Verilator accessors where practical.
- [x] Prevent exceptions from crossing the bridge.

## Generated CXX bridge

- [x] Generate the namespace.
- [x] Generate the opaque DUT declaration.
- [x] Generate construction and lifecycle methods.
- [x] Generate mutable input setters.
- [x] Generate immutable output getters.
- [x] Include the generated adapter header.
- [x] Keep the raw bridge module private.

## Generated Rust wrapper

- [x] Generate a public safe DUT type.
- [x] Hide `UniquePtr`, `Pin`, and raw FFI details.
- [x] Generate `new()`, `eval()`, and `finish()`.
- [x] Generate typed input setters.
- [x] Generate typed output getters.
- [x] Generate rustdoc from metadata.
- [ ] Add narrow lint allowances for generated code.

## Build topology spike

- [x] Write the bridge source under `OUT_DIR`.
- [x] Process it with `cxx_build::bridge()`.
- [x] Compile the same generated bridge through `include!`.
- [x] Make CXX-generated headers visible to the adapter.
- [x] Verify incremental rebuild behavior.
- [x] Verify deterministic generated source.

## Acceptance criteria

- [x] Counter contains no handwritten C++.
- [x] Counter contains no handwritten CXX bridge.
- [x] Generated behavior matches the handwritten version.
- [x] Generated-code snapshot tests exist.
- [ ] Unsupported ports fail before native compilation.

---

# 10. Milestone 5 — Safe generated DUT API

## Goal

Make a generated DUT pleasant to use directly before adding methodology abstractions.

```rust
mod generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/vvm/counter/generated/dut.rs"
    ));
}

use generated::Counter;
```

## API decisions

- [x] Decide whether constructors return a shared or DUT-specific error.
- [x] Decide whether `eval()` can fail.
- [x] Make `finish()` idempotent.
- [x] Decide whether output getters use `&self` or `&mut self`.
- [x] Decide whether explicit simulation time belongs in the initial API.
- [x] Decide whether generated DUTs implement `Debug`.
- [x] Decide whether generated DUTs implement an internal lifecycle trait.

## Lifecycle safety

- [x] Ensure `final()` runs no more than once.
- [x] Ensure dropping an unfinished DUT is safe.
- [x] Ensure explicit `finish()` is safe before drop.
- [x] Define behavior for `eval()` after finish.
- [x] Test construction failure handling.

## Encapsulation

Normal user code must not see:

- [x] `cxx::UniquePtr`
- [x] `Pin<&mut T>`
- [x] Raw CXX modules
- [x] Verilator headers
- [x] Verilator-generated class names
- [x] Unsafe blocks

## Acceptance criteria

- [x] Counter is controlled entirely through safe Rust.
- [x] The wrapper has focused API documentation.
- [x] Raw bridge details are private.
- [x] Lifecycle tests pass.
- [x] The example still uses a manual simulation loop.

---

# 11. Milestone 6 — Minimal pure-Rust VVM core

## Goal

Introduce the smallest useful verification traits around the working generated DUT.

No derive macros and no generalized testbench builder yet.

## Initial traits

```rust
pub trait Dut {
    fn eval(&mut self);
    fn finish(&mut self);
}

pub trait Drive<D>
where
    D: Dut,
{
    fn drive(&self, dut: &mut D);
}

pub trait Sample<D>: Sized
where
    D: Dut,
{
    fn sample(dut: &D) -> Self;
}

pub trait ReferenceModel<S> {
    type Expected;

    fn predict(&mut self, stimulus: &S) -> Self::Expected;
}

pub trait Scoreboard<E, O> {
    type Error;

    fn check(&mut self, expected: E, observed: O) -> Result<(), Self::Error>;
}
```

## `Dut`

- [x] Decide whether methods return `Result`.
- [x] Decide whether the error type is associated.
- [x] Decide whether `finish()` belongs in the trait.
- [x] Ensure a pure Rust mock can implement it.
- [x] Implement it for the generated counter.
- [x] Add mock-DUT tests.

## `Drive`

- [x] Keep it synchronous.
- [x] Use `&self` for immutable stimulus objects.
- [x] Avoid hidden evaluation.
- [x] Document that drive only writes DUT inputs.
- [x] Handwrite the counter stimulus implementation.

## `Sample`

- [x] Keep sampling free of evaluation.
- [x] Document that observations represent current state.
- [x] Handwrite the counter observation implementation.
- [x] Decide whether sampling may fail.

## `ReferenceModel`

- [x] Use an associated expected-output type.
- [x] Allow stateful models.
- [x] Keep it independent of the DUT type.
- [x] Implement a counter reference model.

## `Scoreboard`

- [x] Use structured errors.
- [x] Allow stateful scoreboards.
- [x] Implement exact equality checking.
- [x] Retain expected and observed values in failures.
- [x] Avoid unnecessary global `Debug` bounds.

## Sequences

- [x] Use `Iterator<Item = Stimulus>`.
- [x] Do not create a custom `Sequence` trait yet.
- [x] Implement a finite counter stimulus iterator.
- [x] Include reset and enable transitions.
- [x] Support deterministic iteration.
- [x] Add iterator tests.

## Explicit cycle semantics

Initial synchronous cycle:

1. Drive clock low.
2. Drive stimulus.
3. Evaluate.
4. Drive clock high.
5. Evaluate.
6. Sample observation.
7. Update the reference model.
8. Compare expected and observed.

- [x] Confirm prediction timing relative to the active edge.
- [x] Document reset-cycle behavior.
- [x] Document combinational settling expectations.
- [x] Verify timing with the counter.
- [x] Add an intentional failure test.

## Acceptance criteria

- [x] `vvm-core` has no CXX or Verilator dependency.
- [x] A pure Rust mock DUT test exists.
- [x] Counter stimulus implements `Drive` manually.
- [x] Counter observation implements `Sample` manually.
- [x] Counter reference model works.
- [x] Counter scoreboard detects an intentional mismatch.
- [x] The example still contains an explicit manual run loop.

---

# 12. Milestone 7 — Testbench runner

## Goal

Replace the verified manual loop with a reusable synchronous runner.

Potential shape:

```rust
pub struct Testbench<D, S, R, B, C> {
    dut: D,
    sequence: S,
    reference_model: R,
    scoreboard: B,
    clock: C,
}
```

## Builder API

- [x] Add `Testbench::new`.
- [x] Require a DUT at construction.
- [x] Add sequence configuration.
- [x] Add reference-model configuration.
- [x] Add scoreboard configuration.
- [x] Add clock-driving configuration.
- [x] Use type-state only if it improves errors materially.
- [x] Avoid boxed trait objects initially.
- [x] Preserve concrete generic types.

## Run loop

- [x] Iterate over stimuli.
- [x] Drive each stimulus.
- [x] Perform the defined clock sequence.
- [x] Evaluate at documented points.
- [x] Sample outputs.
- [x] Obtain expected outputs.
- [x] Invoke the scoreboard.
- [x] Track cycles and successful checks.
- [x] Track failures.
- [x] Stop or continue according to configuration.
- [x] Always finish the DUT.

## Results

Suggested shape:

```rust
pub struct TestResult<F> {
    pub cycles: u64,
    pub checks: u64,
    pub failures: Vec<F>,
}
```

- [x] Preserve cycle numbers.
- [x] Preserve expected and observed values.
- [x] Optionally preserve stimuli.
- [x] Add compact and detailed reporting.
- [x] Define example process-exit behavior.

## Failure policy

- [x] Support stop-on-first-failure.
- [x] Support collecting multiple failures.
- [x] Allow a maximum failure count.
- [x] Prevent unbounded diagnostic storage.
- [x] Distinguish simulation errors from check failures.
- [x] Ensure finalization after errors.

## Acceptance criteria

- [x] Counter contains no handwritten simulation loop.
- [x] Successful checks are reported.
- [x] Intentional failures include cycle-aware diagnostics.
- [x] The runner is tested with a pure Rust mock DUT.
- [x] No procedural macros are used yet.

---

# 13. Milestone 8 — Derive macros

## Goal

Generate repetitive `Clock`, `Drive`, and `Sample` implementations after their handwritten forms are proven.

## Intended API

```rust
#[derive(Clone, Debug, vvm::Clock)]
#[vvm(dut = crate::dut::Counter, clock = "clk")]
struct CounterClock;
```

```rust
#[derive(Clone, Debug, vvm::Drive)]
#[vvm(dut = crate::dut::Counter)]
struct CounterStimulus {
    #[vvm(port)]
    reset_n: bool,

    #[vvm(port)]
    enable: bool,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, vvm::Sample)]
#[vvm(dut = crate::dut::Counter)]
struct CounterObservation {
    #[vvm(port)]
    count: u8,
}
```

Explicit rename:

```rust
#[vvm(port = "reset_n")]
reset: bool,
```

## Source layout

```text
vvm-macros/src/
├── lib.rs
├── attrs.rs
├── error.rs
├── drive/
│   ├── mod.rs
│   ├── input.rs
│   └── expand.rs
├── clock/
│   ├── mod.rs
│   ├── input.rs
│   └── expand.rs
└── sample/
    ├── mod.rs
    ├── input.rs
    └── expand.rs
```

## Shared attribute parsing

- [x] Parse `#[vvm(dut = path)]`.
- [x] Parse `#[vvm(port)]`.
- [x] Parse `#[vvm(port = "hdl_name")]`.
- [x] Reject duplicate DUT attributes.
- [x] Reject duplicate port attributes.
- [x] Reject unknown options.
- [x] Preserve useful spans.
- [x] Avoid panics for malformed input.

## `Clock` derive

- [x] Accept unit structs.
- [x] Reject enums and non-unit structs.
- [x] Require a DUT path.
- [x] Require a clock port name.
- [x] Support rising and falling edge selection.
- [x] Preserve generics and where clauses.
- [x] Use hygienic VVM trait paths.
- [x] Produce readable expanded code.

## `Drive` derive

- [x] Accept named-field structs.
- [x] Reject tuple structs and enums.
- [x] Require a DUT path.
- [x] Generate one setter call per mapped field.
- [x] Use field names by default.
- [x] Support explicit HDL names.
- [x] Preserve generics and where clauses.
- [x] Use hygienic VVM trait paths.
- [x] Produce readable expanded code.

## `Sample` derive

- [x] Accept named-field structs.
- [x] Reject tuple structs and enums.
- [x] Require a DUT path.
- [x] Generate one getter call per mapped field.
- [x] Construct observations with named fields.
- [x] Support explicit HDL names.
- [x] Preserve generics and where clauses.
- [x] Produce readable expanded code.

## Direction and type checking

Rely initially on generated DUT method availability:

- Driving an output calls a nonexistent setter.
- Sampling an input calls a nonexistent getter.
- Incompatible field types fail ordinary Rust type checking.

- [x] Confirm resulting diagnostics are understandable.
- [x] Add custom macro diagnostics only where they help.
- [x] Avoid duplicating DUT metadata in macro input initially.

## `trybuild` tests

Pass cases:

- [x] Basic `Clock`.
- [x] Basic `Drive`.
- [x] Basic `Sample`.
- [x] Explicit port rename.
- [x] Generic input where valid.
- [x] Coexistence with unrelated derives.

Fail cases:

- [x] Missing clock DUT attribute.
- [x] Missing clock port.
- [x] Invalid clock edge.
- [x] Invalid clock target type.
- [x] Missing DUT attribute.
- [x] Enum derives `Drive`.
- [x] Tuple struct derives `Sample`.
- [x] Unknown `vvm` option.
- [x] Duplicate port metadata.
- [x] Invalid port syntax.
- [x] Driving an output.
- [x] Sampling an input.
- [x] Incompatible field type.
- [x] Missing setter or getter.

## Acceptance criteria

- [x] Handwritten counter `Clock` implementation is removed.
- [x] Handwritten counter `Drive` implementation is removed.
- [x] Handwritten counter `Sample` implementation is removed.
- [x] Runtime behavior is unchanged.
- [x] Compile-fail tests verify diagnostics.
- [x] Macro entry points remain thin.
- [x] Macro internals are tested through `proc_macro2`.

---

# 14. Milestone 9 — Public facade

## Goal

Expose only stable, proven APIs.

Possible contents:

```rust
pub use vvm::{
    Clock,
    Drive,
    Dut,
    ReferenceModel,
    Sample,
    Scoreboard,
    TestResult,
    Testbench,
};
```

## Re-export checklist

- [x] Re-export stable core traits.
- [x] Re-export stable result types.
- [x] Re-export stable derive macros.
- [x] Add facade-level crate documentation.
- [x] Add a minimal prelude.
- [x] Verify derive and trait name coexistence.
- [x] Avoid re-exporting internal parser or codegen types.
- [x] Keep `vvm-build` as a direct build dependency.
- [x] Remove the unused `vvm-ffi` crate.

## DUT inclusion convenience

Potential API:

```rust
vvm::include_dut!(counter);
```

- [x] Decide whether inclusion belongs in `vvm` or `vvm-build`.
- [x] Define generated file naming conventions.
- [x] Support multiple DUTs in one crate.
- [x] Prevent module-name collisions.
- [x] Keep generated implementation modules private by default.
- [x] Document manual `include!` fallback.

## Acceptance criteria

- [x] Example user code imports from `vvm`.
- [x] Example `build.rs` imports from `vvm-build`.
- [x] Ordinary users need no internal crates directly.
- [x] The facade exposes no unstable implementation details.
- [x] Public rustdoc shows an end-to-end counter example.

---

# 15. Milestone 10 — Tracing, reporting, and standard test harness

## Waveform tracing

Implement VCD first, then FST.

- [ ] Add trace configuration to `DutBuilder`.
- [ ] Enable matching Verilator flags.
- [ ] Generate trace ownership in the adapter.
- [ ] Expose safe trace initialization.
- [ ] Dump at defined simulation times.
- [ ] Flush and close on finish.
- [ ] Add a traced counter example.
- [ ] Document time units and dump ordering.

## Simulation time

- [ ] Introduce an explicit time type.
- [ ] Decide whether time is ticks or typed units.
- [ ] Increment time deterministically.
- [ ] Pass time to waveform dumping.
- [ ] Keep cycle count distinct from simulation time.
- [ ] Prepare for timing-enabled models.

## Diagnostics

- [ ] Add compact pass/fail summaries.
- [ ] Add detailed mismatch reports.
- [ ] Include cycle and simulation time.
- [ ] Include stimulus where available.
- [ ] Include expected and observed values.
- [ ] Add field-level comparisons later.
- [ ] Keep report storage bounded.

## Deterministic randomization

- [ ] Select an RNG crate.
- [ ] Require explicit or recorded seeds.
- [ ] Print the seed in every run.
- [ ] Include the seed in failure reports.
- [ ] Support replaying a failing run.
- [ ] Avoid hidden global RNG state.

## Test registry

- [ ] Define a test descriptor.
- [ ] Define a registry.
- [ ] Reject duplicate names.
- [ ] Decide whether registration uses a procedural macro.
- [ ] Keep registry use independent of Cargo test discovery.

## Standard Rust test integration

Target:

```text
cargo test counter_random

VVM_SEED=0x1234 cargo test counter_random

VVM_CYCLES=10000 cargo nextest run counter_random

VVM_TRACE_DIR=target/custom-vvm-traces cargo test counter_smoke
```

- [ ] Make `#[vvm::test]` expand to an ordinary Rust `#[test]`.
- [ ] Keep the source function name as the visible test name.
- [ ] Move the typed implementation into a hidden helper.
- [ ] Preserve `cfg`/`cfg_attr` gating across generated items.
- [ ] Propagate `#[ignore]` to the visible wrapper.
- [ ] Reject `#[should_panic]`.
- [ ] Reject manual `#[test]` on `#[vvm::test]` functions.
- [ ] Execute descriptors through one shared runtime path.
- [ ] Replace CLI flags with environment configuration.
- [ ] Support `VVM_SEED`, `VVM_REPLAY`, `VVM_CYCLES`, and `VVM_TRACE_DIR`.
- [ ] Generate default trace paths under one per-run root.
- [ ] Remove the dedicated CLI runner from the facade.
- [ ] Remove the registry macro from standard example usage.

## Acceptance criteria

- [ ] `cargo test` discovers VVM tests directly.
- [ ] `cargo nextest run` discovers the same VVM tests directly.
- [ ] A test can be filtered by its original Rust function name.
- [ ] A deterministic replay token is displayed and replayable.
- [ ] A waveform is generated for trace-capable tests.
- [ ] Failure reports are reproducible.
- [ ] The facade no longer exposes a dedicated CLI runner.

---

# 16. Milestone 11 — Wider HDL feature support

Implement only after the MVP is stable.

## Signed signals

- [ ] Parse signedness.
- [ ] Map widths to signed Rust integer types.
- [ ] Define sign-extension behavior.
- [ ] Test boundary widths.
- [ ] Add a signed arithmetic example.

## Widths above 64 bits

- [ ] Design `Bits<const N: usize>`.
- [ ] Decide word and bit order.
- [ ] Generate CXX-compatible transfer helpers.
- [ ] Avoid heap allocation where practical.
- [ ] Add 128-bit and 256-bit tests.

## Multiple clocks

- [x] Represent named clocks.
- [x] Define independent periods.
- [x] Define same-time ordering.
- [x] Add scheduler support.
- [x] Add a multi-clock example.

## Inout ports

### 11.6.0 — Direct Verilator model-member ABI

#### 11.6.0A — Remove accessor-based model generation

- [x] Stop requesting `--emit-accessors` for final model generation.
- [x] Reserve both accessor-control raw arguments.

#### 11.6.0B — Rename the internal model-member contract

- [x] Resolve each ordinary HDL port to its public Verilator model member.

#### 11.6.0C — Migrate generated C++ port transfers

- [x] Read and write public top-level model members in the private adapter implementation.

### Direct Verilator model-member ABI

- VVM does not request `--emit-accessors`.
- Generated C++ adapters read and write Verilator public top-level model members.
- Verilator-specific signal access remains confined to the private generated adapter implementation.
- Rust APIs, CXX bindings, and runtime traits remain independent of the native signal-access strategy.
- VVM supports exactly one model-port ABI.
- The migration enables later use of `--pins-inout-enables`.

### 11.6.1 — Inout metadata and model-generation contract

- [x] Adopt one direct Verilator model-member ABI.
- [x] Preserve semantic inout metadata.
- [x] Accept plain packed-scalar inouts.
- [x] Reject aggregate inouts precisely.
- [x] Detect inout presence from normalized metadata.
- [x] Enable `--pins-inout-enables` automatically.
- [x] Verify direct `<port>`, `<port>__en`, and `<port>__out` members.
- [x] Add generated safe inout API.
- [x] Add external resolution and contention handling.
- [x] Add vertical tri-state example.

### Inout model-generation contract

- Metadata preserves one semantic `PortDirection::Inout`.
- Final model generation uses `--pins-inout-enables`.
- The direct Verilator model exposes `<port>`, `<port>__en`, and `<port>__out`.
- Generated VVM-facing methods expose raw inout components.
- External resolution remains caller-owned.

### 11.6.2 — Inout generated API

- [x] Separate externally resolved input, output enable, and output value.
- [x] Add `InoutState<Value, Enable>`.
- [x] Generate component-level C++ adapter methods.
- [x] Generate component-level CXX bindings.
- [x] Generate safe Rust component methods.
- [x] Generate `set_<port>` compatibility aliases.
- [x] Generate `<port>()` composite snapshots.
- [x] Integrate inouts naturally with `Drive` and `Sample`.
- [x] Add external resolution and contention handling.
- [x] Add the full vertical tri-state example.

### Generated inout API

- `set_<port>_input(value)` presents an externally resolved value to the DUT.
- `set_<port>(value)` is a Drive-compatible alias.
- `<port>_input()` returns the currently presented input.
- `<port>_output_enable()` returns the per-bit DUT drive mask.
- `<port>_output_value()` returns the DUT-proposed value.
- `<port>()` returns `InoutState<Value, Enable>`.
- None of these methods evaluates the DUT, advances time, or resolves drivers automatically.

### 11.6.3 — Tri-state example and documentation

- [x] Add caller-owned external resolution.
- [x] Add exact per-bit contention detection.
- [x] Add an explicit floating-bit policy.
- [x] Add bounded combinational settling.
- [x] Add a vertical tri-state bus example.
- [x] Add VCD coverage.
- [x] Document two-state limitations.
- [x] Document open-drain adaptation.

### Inout resolution semantics

- VVM exposes raw inout components and does not prescribe an electrical resolution policy.
- The caller combines DUT and external driver proposals.
- Driver values are meaningful only where their enable mask is set.
- Floating-bit and contention policies are explicit.
- Successfully resolved values are written through `set_<port>_input`.
- Combinational feedback is settled through bounded repeated evaluation at one logical simulation time.
- Two-state Verilator execution does not expose Rust-visible `X` or `Z` values.

## Arrays and aggregates

- [ ] Parse unpacked dimensions.
- [ ] Design array accessors.
- [ ] Parse packed structs and unions.
- [ ] Decide flattened versus typed representations.
- [ ] Verify bit layouts carefully.
- [ ] Add integration tests.

## Timing-enabled models

- [x] Add a distinct timing scheduler.
- [x] Support `eventsPending()`.
- [x] Support `nextTimeSlot()`.
- [x] Define time advancement.
- [x] Integrate waveform dumping.
- [x] Add a delay-based example.
- [x] Preserve cycle-based mode.

### Timing-enabled model semantics

- `TimedDut` exposes simulator-owned delayed-event times.
- `TimingScheduler` performs one initialization evaluation.
- Each future absolute event time is converted to one positive relative advance.
- The DUT is evaluated exactly once at each processed slot.
- Waveform dumping remains part of normal DUT evaluation.
- `TimingScheduler` does not finalize the DUT.
- Cycle-driven `Testbench` and internal timing scheduling remain separate.
- Same-time and `#0` scheduling remain unsupported.

### 11.7 — Rust-native functional coverage

#### 11.7.1 — Coverage primitives

- [x] Add typed `Coverpoint<T>`.
- [x] Add normal, ignore, and illegal bins.
- [x] Add exact-value bins.
- [x] Add value-set bins.
- [x] Add inclusive-range bins.
- [x] Add configurable non-zero hit thresholds.
- [x] Add explicit sampling.
- [x] Define illegal, ignore, normal, and unmatched precedence.
- [x] Record all matching bins within the selected category.
- [x] Add deterministic coverpoint-local `BinId`s.
- [x] Add live bin and coverpoint inspection.
- [x] Add exact `CoverageRatio`.
- [x] Add uncovered-bin inspection.
- [x] Add structured construction errors.
- [x] Add structured sampling errors.
- [x] Guarantee atomic counter updates on overflow.
- [x] Add comprehensive matcher, builder, sampling, metric, error, and atomicity tests.
- [x] Document the primitive functional-coverage API.

#### 11.7.2 — Two-way cross coverage

- [x] Add exact coverpoint-sample provenance.
- [x] Validate exact source coverpoint instances.
- [x] Add complete two-way normal-bin crosses.
- [x] Add deterministic row-major cross-bin IDs.
- [x] Preserve source bin IDs and names.
- [x] Add configurable cross-bin hit thresholds.
- [x] Add a default cardinality guard.
- [x] Add an explicit cardinality override.
- [x] Detect cardinality arithmetic overflow.
- [x] Cross overlapping normal bins through Cartesian products.
- [x] Skip crosses for ignored and unmatched samples.
- [x] Preserve source sample dispositions.
- [x] Guarantee atomic cross-counter updates.
- [x] Reuse exact `CoverageRatio`.
- [x] Add covered and uncovered cross-bin inspection.
- [x] Document two-way cross semantics.
#### 11.7.3 — Coverage groups and instances

- [x] Add stable group-definition and instance-path metadata.
- [x] Add hierarchical instance-path validation.
- [x] Add read-only type-erased coverage-item views.
- [x] Add the object-safe `CoverageGroup` trait.
- [x] Add deterministic item visitation.
- [x] Validate unique item names and cross source membership.
- [x] Add exact aggregate group metrics.
- [x] Preserve independent per-instance counters.

#### 11.7.4 — Per-test coverage sessions

- [x] Add immutable coverpoint-bin, cross-bin, item, and group snapshots.
- [x] Add owned per-test `CoverageSession` snapshots and exact flat aggregation.
- [x] Enforce unique group instance paths and atomic capture.
- [x] Add context-aware VVM test execution while preserving legacy descriptors.
- [x] Add `&mut TestContext` support to `#[vvm::test]`.
- [x] Attach completed coverage snapshots to `TestRun`.
- [x] Add versioned JSON persistence and definition fingerprints.

Functional coverage primitive semantics:

- Coverage state is owned explicitly by ordinary Rust values.
- Sampling occurs only through `Coverpoint::sample`.
- No global or thread-local coverage database exists.
- Illegal bins take precedence over ignore and normal bins.
- Ignore bins take precedence over normal bins.
- All matching bins within the selected category increment.
- Illegal hits are recorded before an error is returned.
- Ignore and illegal bins do not contribute to coverage completion.
- Normal bins become covered after reaching their configured hit count.
- Primitive coverage remains an exact integer ratio.
- Percentages are deferred to the reporting layer.
- Matchers remain declarative for later fingerprinting, persistence, merging, reporting, and UCIS export.

Two-way cross semantics:

- Crosses bind to exact coverpoint instances.
- Crosses consume `CoverpointSample` normal-bin identities.
- Cross sampling remains explicit.
- Only normal bins participate.
- Overlapping bins form a Cartesian product.
- Cross-bin ordering is deterministic and row-major.
- Ignored or unmatched axes skip the cross.
- Cross cardinality is bounded during construction.
- Counter-overflow failures mutate nothing.
- Cross coverage remains an exact integer ratio.

Coverage-group semantics:

- Coverage groups are ordinary user-defined Rust structs.
- Concrete structs retain ownership of typed coverpoints and crosses.
- Sampling remains concrete and user-defined.
- Item visitation is deterministic and read-only.
- Cross membership uses exact process-local coverpoint identity.
- Group coverage is a flat exact ratio over all item bins.
- Instance merging and definition compatibility are deferred.

Per-test coverage-session semantics:

- Live coverage remains owned by concrete user-defined group structs.
- Sessions capture immutable owned snapshots and never retain live group references.
- Capture freezes group state at that call and preserves capture order.
- Group instance paths are unique within one test session.
- Session aggregation is an exact flat bin ratio.
- `TestContext` owns the effective configuration and coverage session.
- Existing `&TestRunConfig` tests remain supported; context-aware tests use `&mut TestContext`.
- `TestRun` owns the completed optional coverage snapshot.
- Artifact persistence, deterministic merging, and reporting are implemented explicitly offline.

#### 11.7.5 — Versioned JSON coverage artifacts

- [x] Add strict schema-v1 JSON serialization and validation.
- [x] Add SHA-256 structural definition fingerprints.
- [x] Add atomic same-directory artifact writes.
- [x] Persist captured test-session coverage through the standard facade bridge.
- [x] Add `VVM_COVERAGE_DIR` output-root configuration.
- [x] Document schema-v1 and fingerprint limitations.

#### 11.7.6 — Deterministic coverage merging

- [x] Add explicit test-status inclusion policies.
- [x] Default to passed-only coverage contribution.
- [x] Preserve metadata for excluded artifacts.
- [x] Merge group instances by hierarchical instance path.
- [x] Require matching definition fingerprints and structural data.
- [x] Sum runtime counters with checked arithmetic.
- [x] Recompute coverage after merging raw hits.
- [x] Produce deterministic results independent of input order.
- [x] Order merged groups lexicographically by instance path.
- [x] Add the immutable `CoverageMerge` model and strict merged JSON schema.
- [x] Add atomic merged-document persistence and explicit file merge APIs.
- [x] Preserve per-test schema-v1 behavior.
- [x] Document merge semantics and status policy.

Coverage merge semantics:

- Per-test artifacts remain immutable inputs.
- Merge contribution is controlled by an explicit status policy; passed-only is the default.
- Groups merge by exact hierarchical instance path; different paths remain independent.
- Compatible groups require equal names, revisions, fingerprints, and ordered structure.
- Raw runtime counters are summed with checked arithmetic and coverage is recomputed.
- Input order cannot affect merged results; groups use lexicographic instance-path order.
- Excluded artifacts remain provenance metadata.
- Merging is explicit offline post-processing; test execution has no shared merged database.

#### 11.7.7 — Coverage reporting and CI metrics

- [x] Add deterministic fixed-point coverage percentages.
- [x] Prevent incomplete coverage from displaying as 100%.
- [x] Add configurable bin-detail reporting.
- [x] Add deterministic plain-text reports.
- [x] Add self-contained HTML reports.
- [x] Preserve exact counts alongside percentages.
- [x] Report input-artifact provenance and inclusion state.
- [x] Report group, coverpoint, and cross summaries.
- [x] Surface uncovered normal and cross bins.
- [x] Surface hit illegal bins.
- [x] Add stable GitLab-compatible metric output.
- [x] Add a documented GitLab coverage regex.
- [x] Document reporting and CI integration.

Coverage reporting semantics:

- Reports consume one validated deterministic `CoverageMerge`.
- Exact integer ratios remain authoritative.
- Percentages use deterministic fixed-point formatting.
- Only complete ratios display `100.00%`.
- Plain-text and HTML reports share percentage and bin-selection logic.
- Uncovered reports also surface hit illegal bins.
- Text output ends with one stable GitLab-compatible metric line.
- HTML output is self-contained and contains no JavaScript or external network resources.
- Reporting is explicit offline post-processing.
- Test execution does not generate reports automatically.

#### 11.7.8 — Vertical functional-coverage example

- [x] Add synchronous per-cycle testbench observation.
- [x] Preserve existing `Testbench::run` behavior.
- [x] Add typed counter coverpoints.
- [x] Add counter two-way crosses.
- [x] Sample only successfully executed DUT cycles.
- [x] Capture coverage through `TestContext`.
- [x] Preserve smoke, randomized, tracing, replay, and failure behavior.
- [x] Emit isolated per-test JSON artifacts.
- [x] Merge compatible smoke and randomized coverage.
- [x] Generate deterministic merged JSON.
- [x] Generate deterministic plain-text and HTML reports.
- [x] Emit the GitLab-compatible metric.
- [x] Add an example-specific offline postprocessor.
- [x] Add a one-command local workflow.
- [x] Add comprehensive observer, coverage, integration, post-processing, and workflow tests.
- [x] Document the complete functional-coverage pipeline.

Vertical counter coverage semantics:

- Coverage observes successfully sampled testbench transactions synchronously.
- Observation cannot drive, evaluate, independently sample, or advance the DUT.
- Coverage does not alter reference-model or scoreboard behavior.
- Counter coverage remains an ordinary user-owned Rust struct.
- Reset is an ignored operation for operation-based crosses.
- Valid test traffic never samples the explicit illegal operation.
- Smoke and randomized tests capture the same definition at the same hierarchical instance path.
- Standard test execution emits isolated per-test artifacts.
- The counter postprocessor performs explicit offline merging.
- Text and HTML reports consume only the validated merged model.
- The GitLab metric is the final line of the text report.

#### 11.7.9 — Ergonomic coverage integration

- [x] Add typed coverage models, validated instances, and framework errors.
- [x] Add coverage testbench typestate and covered execution.
- [x] Generate construction, visitation, sampling, and cross routing with `Coverage`.
- [x] Add declared coverage capability and missing-capture diagnostics.
- [x] Migrate the counter example to the ergonomic workflow.
- [x] Preserve per-test persistence and explicit offline merge/reporting.

Ergonomic coverage semantics:

- Coverage intent remains in ordinary Rust builder and extractor functions.
- Derived models are wrapped in `CoverageInstance<M>` and validate before use.
- Covered testbenches sample at the deterministic low-level observer point.
- Coverage failures are deferred through `TestContext`; partial coverage survives.
- Declared coverage detects completed testbench runs that capture nothing.
- Suite-wide merging and reporting remain explicit offline orchestration.

#### 11.7.10 — Coverage execution orchestration

- [x] Add the `cargo-vvm` Cargo subcommand and `cargo vvm coverage` workflow.
- [x] Support Cargo test, nextest run, optional toolchains, and manifest paths.
- [x] Isolate artifacts, merge through core coverage models, and render reports atomically.
- [x] Preserve live child output and original failed-test status after post-processing.
- [x] Replace the counter postprocessor and document local and GitLab workflows.

Coverage orchestration semantics:

- `cargo-vvm` owns one isolated suite coverage-run directory.
- Test processes remain ordinary Cargo test processes and emit isolated artifacts.
- Child output remains live; post-processing runs after successful and failed tests.
- Nextest retries are disabled until artifacts identify retry attempts.
- Discovery is non-recursive and merging/reporting reuse strict core models.
- Text output ends with the existing GitLab-compatible metric.

### Future — DPI interoperability

DPI support is deferred until a concrete third-party HDL, native-model, legacy
verification-environment, or commercial-simulator interoperability requirement
justifies the additional FFI, scope, callback, and lifecycle surface.

---

# 17. Milestone 12 — Pre-release cleanup and polish

## Goal

Prepare VVM for its first real public release by stabilizing its architecture, public API, tests, examples, documentation, performance baselines, packaging, compatibility policy, and release process.

Milestone 12 is in progress (12.2 complete) and is primarily a stabilization milestone.

It should not introduce another broad feature wave. Work that is not required for a credible `v0.2.0` release should be explicitly deferred rather than allowed to expand the release scope indefinitely.

The milestone should produce:

```text
coherent public API
    ↓
consistent internal architecture
    ↓
documented error and diagnostic policy
    ↓
structured unit, integration, compile-time, and end-to-end tests
    ↓
curated realistic examples
    ↓
complete mdBook and rustdoc documentation
    ↓
repeatable benchmark baselines
    ↓
verified crates.io packages
    ↓
release candidate
    ↓
VVM v0.2.0
```

## Release principles

* [ ] Freeze the `v0.2.0` feature scope before beginning broad cleanup.
* [ ] Classify every unfinished earlier-roadmap task as either release-blocking or post-`0.2.0`.
* [ ] Do not require every aspirational Milestone 11 feature for `v0.2.0`.
* [ ] Do not add new public abstractions solely to make the cleanup appear comprehensive.
* [ ] Prefer a deliberate pre-`0.2.0` breaking cleanup over carrying accidental API decisions into the first release.
* [ ] Make every public item have one canonical documented path.
* [ ] Test packaged consumer workflows rather than only workspace path dependencies.
* [ ] Establish compatibility and performance baselines before publishing `v0.2.0`.
* [ ] Publish a release candidate before the final release.
* [ ] Treat documentation, packaging, and release automation as release functionality rather than optional polish.

## Required implementation order

Complete the milestone in this order:

```text
12.1 Public API and facade stabilization
12.2 Error and diagnostic consistency
12.3 Test architecture and coverage hardening
12.4 Example curation and realistic showcases
12.5 mdBook, rustdoc, and GitLab Pages
12.6 Benchmark suite and performance baselines
12.7 Packaging, compatibility, and release engineering
12.8 Safety, dependency, and final release audit
12.9 v0.2.0 release candidate
12.10 v0.2.0 release
```

Tests, examples, documentation, and benchmarks must target the stabilized API rather than being written against paths that are about to change.

---

## 12.1 — Public API and facade stabilization

### Objective

Replace the flat facade with a coherent domain-oriented public API and establish the public compatibility surface for `v0.2.0`.

### Completion status

* [x] Inventory all current facade exports.
* [x] Establish canonical facade modules.
* [x] Define the root-level macro policy.
* [x] Define the prelude inclusion policy.
* [x] Rename the private harness module.
* [x] Establish `vvm::__private` as generated-code ABI.
* [x] Migrate all procedural macro generated paths.
* [x] Migrate all `vvm-build` generated paths.
* [x] Add domain-oriented facade modules.
* [x] Remove the flat root type re-export surface.
* [x] Remove accidental facade exports.
* [x] Migrate all examples and fixtures.
* [x] Migrate all documentation and doctests.
* [x] Add focused facade integration tests.
* [x] Add generated-code path tests.
* [x] Produce and review the provisional public API inventory.
* [x] Document the public API architecture.

### Release-scope freeze

* [ ] Review all unfinished Milestone 11 tasks.
* [ ] Identify tasks required for a credible `v0.2.0`.
* [ ] Move non-blocking HDL features into an explicitly post-`0.2.0` roadmap.
* [ ] Define a feature freeze for Milestone 12.
* [ ] Require release-scope justification for any new public API added during cleanup.

### Canonical facade modules

Create canonical public modules conceptually resembling:

```rust
vvm::coverage
vvm::dut
vvm::packed
vvm::random
vvm::report
vvm::test
vvm::testbench
vvm::timing
```

The exact module boundaries must be derived from the implemented domain model rather than chosen only for visual symmetry.

* [ ] Inventory every current root re-export.
* [ ] Assign every public type, trait, function, constant, and error to one domain.
* [ ] Create public facade modules with focused rustdoc landing pages.
* [ ] Give every public item one canonical documented path.
* [ ] Remove accidental or implementation-only exports.
* [ ] Keep generated-code-only exports under `__private`.
* [ ] Ensure generated code uses stable absolute facade paths.
* [ ] Update all workspace crates, examples, fixtures, tests, and documentation to canonical paths.
* [ ] Add public-path integration tests using only the `vvm` facade.

### Root-level API policy

Keep root-level exports only where they materially improve normal Rust usage.

Likely root-level exports:

```rust
vvm::Clock
vvm::Coverage
vvm::Drive
vvm::Sample
vvm::test
vvm::include_dut!
vvm::prelude
```

These derive and attribute macros benefit from short canonical paths.

* [ ] Decide which macros and derives remain at the root.
* [ ] Decide whether any essential test-authoring types remain at the root.
* [ ] Remove the broad flat type re-export list.
* [ ] Do not retain duplicate root aliases merely to avoid a pre-release migration.
* [ ] Document the deliberate breaking API cleanup in the changelog.

### Domain placement

Prefer traits beside their domain rather than one catch-all `traits` module.

Conceptual organization:

```rust
vvm::dut::{
    Clock,
    Drive,
    Dut,
    Sample,
    TimedDut,
    TraceableDut,
}

vvm::testbench::{
    ExactScoreboard,
    FailurePolicy,
    ObservedCycle,
    ReferenceModel,
    Scoreboard,
    Testbench,
}

vvm::coverage::{
    Bin,
    CoverageInstance,
    CoverageMerge,
    CoverageModel,
    CoverageReport,
    Coverpoint,
    Cross2,
}
```

* [ ] Keep trait and related implementation types discoverable together.
* [ ] Avoid modules containing only arbitrary type-category groupings.
* [ ] Avoid exposing `vvm-core` source-module organization accidentally through the facade.
* [ ] Verify rustdoc module indexes remain understandable.

### Prelude policy

The prelude should contain only frequently required test-authoring items.

Candidate contents:

```rust
Clock
Drive
Dut
ReferenceModel
Sample
Scoreboard
Testbench
TestContext
```

Potential optional contents:

```rust
CoverageModel
Randomize
```

* [ ] Define and document the prelude inclusion policy.
* [ ] Remove persistence, snapshot, report, builder, and specialized error types.
* [ ] Ensure derives and traits with matching names remain usable together.
* [ ] Add compile tests for ordinary prelude usage.
* [ ] Add documentation showing explicit imports for advanced APIs.

### Published-crate boundaries

Classify the workspace crates:

```text
vvm-rs
    Supported primary user-facing library.

vvm-build
    Supported user-facing build-script library.

cargo-vvm
    Supported user-facing Cargo subcommand.

vvm-core
    Published implementation foundation and advanced API.
    Ordinary users should prefer the facade.

vvm-macros
    Published procedural-macro implementation dependency.
    Ordinary users should not depend on it directly.
```

* [ ] Confirm which crates are published.
* [ ] Confirm which crates are documented as supported direct entry points.
* [ ] Review every cross-crate public dependency.
* [ ] Prevent users from requiring `vvm-core` or `vvm-macros` directly for ordinary workflows.
* [ ] Document support expectations for each package.

### Extensibility audit

* [ ] Review every public trait for intended downstream implementation.
* [ ] Seal traits that exist only for generated code or internal coordination.
* [ ] Verify public traits do not expose private implementation concepts.
* [ ] Review blanket implementations for future coherence conflicts.
* [ ] Review default generic parameters for long-term compatibility.
* [ ] Review public constructors for invariant bypasses.
* [ ] Add `#[non_exhaustive]` to extensible public enums and structs where appropriate.
* [ ] Avoid `#[non_exhaustive]` where exhaustive matching is an intentional stable contract.

### Public API baseline

* [ ] Generate a machine-readable public API inventory.
* [ ] Review it manually for accidental exports.
* [ ] Store the accepted `v0.2.0` API baseline.
* [ ] Add `cargo-public-api` or equivalent contributor tooling.
* [ ] Add `cargo-semver-checks` for post-release compatibility checks.
* [ ] Document how intentional breaking changes are reviewed after `v0.2.0`.

### Acceptance criteria

* [ ] The facade no longer re-exports nearly every public item at its root.
* [ ] Public APIs are grouped into coherent documented modules.
* [ ] Every public item has one canonical path.
* [ ] The prelude is intentionally small.
* [ ] Ordinary users require only `vvm`, `vvm-build`, and optionally `cargo-vvm`.
* [ ] Generated macro paths remain hygienic.
* [ ] All examples and documentation use the stabilized API.
* [ ] A reviewed public API baseline exists.

---

## 12.2 — Error and diagnostic consistency

### Objective

Establish one consistent architecture for public errors, internal errors, compiler diagnostics, test diagnostics, and CLI failures.

### Error inventory

* [x] Inventory every error type in every workspace crate.
* [x] Record its visibility, domain, source module, implementation style, and consumers.
* [x] Identify duplicated or overlapping error concepts.
* [x] Identify errors exposed publicly only by accident.
* [x] Identify stringly typed failures that should become structured variants.
* [x] Identify error enums whose variants expose unstable internal types.

### Source-layout policy

Use one consistent rule:

```text
small domain with one compact error:
    error may remain in the domain module

large subsystem or several related errors:
    subsystem/error.rs

avoid:
    arbitrary mixtures of error.rs,
    foo_error.rs,
    and unrelated embedded enums
```

* [x] Define the policy in the developer guide.
* [x] Normalize comparable subsystems.
* [x] Avoid mechanical file moves that do not improve domain clarity.
* [x] Keep error definitions close enough to their domain to remain discoverable.

### Naming policy

Namespacing permits concise public names.

Conceptual facade API:

```rust
vvm::coverage::BuildError
vvm::coverage::SampleError
vvm::coverage::MergeError
vvm::coverage::PersistenceError

vvm::testbench::SimulationError
vvm::test::RegistryError

vvm_build::BuildError
cargo_vvm::Error
```

* [x] Prefer clear names within canonical modules.
* [x] Avoid unnecessarily repeating the module name in every type.
* [x] Preserve more specific names where ambiguity would remain.
* [x] Document all intentional public renames in the developer API documentation.

### Implementation policy

* [x] Use `thiserror` for ordinary structured error enums.
* [x] Handwrite `Display` only when deriving cannot express the intended stable output.
* [x] Handwrite `Error` only where custom source behavior is required.
* [x] Remove redundant manual implementations.
* [x] Preserve deterministic and tested error messages.
* [x] Do not require callers to parse `Display`.
* [x] Avoid using boxed dynamic errors in normal framework APIs.
* [x] Preserve concrete source chains.

### Public error contract

Every public error should provide, where relevant:

```text
deterministic Display
meaningful Error::source
path accessors
item or signal accessors
stage or operation accessors
status accessors
non-lossy nested source
```

* [x] Audit every public error for structured inspection.
* [x] Add `#[non_exhaustive]` where new variants are expected.
* [x] Review source types for public stability.
* [x] Ensure error messages do not leak irrelevant temporary paths or implementation details.
* [x] Ensure non-UTF-8 paths remain representable where applicable.

### Panic and invariant audit

* [x] Audit production `panic!`, `unwrap`, `expect`, indexing, and unreachable assumptions.
* [x] Replace recoverable panics with structured errors.
* [x] Document genuinely impossible internal invariants.
* [x] Keep panic behavior away from FFI boundaries.
* [x] Ensure malformed user input cannot panic procedural macros.
* [x] Ensure malformed persisted artifacts cannot panic readers.
* [x] Ensure CLI input and filesystem failures remain structured.

### Diagnostic layers

Distinguish:

```text
Rust compiler diagnostics
    Procedural macro misuse and type errors.

Build diagnostics
    Verilator, compiler, metadata, and native-link failures.

Simulation diagnostics
    DUT driving, evaluation, sampling, timing, and scoreboard failures.

Coverage diagnostics
    Definition, sampling, capture, persistence, and merge failures.

Command diagnostics
    cargo-vvm configuration and orchestration failures.
```

* [x] Ensure each layer has a clear owner.
* [x] Avoid converting structured failures into strings prematurely.
* [x] Preserve original verification failures when framework diagnostics are appended.
* [x] Document diagnostic ordering and aggregation.

### Tests

* [x] Test every public error variant.
* [x] Test every nested `source()`.
* [x] Test structured accessors.
* [x] Test deterministic `Display`.
* [x] Test combined verification and framework diagnostics.
* [x] Test non-UTF-8 paths on supported platforms.
* [x] Test that malformed external data returns errors rather than panicking.

### Acceptance criteria

* [x] Error placement follows one documented policy.
* [x] Comparable errors use consistent names and implementation style.
* [x] Public errors are structurally inspectable.
* [x] Manual `Display` and `Error` implementations are exceptional and justified.
* [x] Recoverable user failures do not panic.
* [x] Error and diagnostic tests cover every public variant.

---

## 12.3 — Test architecture and coverage hardening

### Objective

Replace the current ad hoc test layout with an explicit testing strategy covering private invariants, public API contracts, macro behavior, package integration, and complete external workflows.

### Testing taxonomy

Adopt the following categories.

#### Unit tests

Use unit tests for:

```text
private invariants
small parsers
state transitions
checked arithmetic
overflow behavior
error formatting
internal scheduling
```

* [ ] Keep short tests inline where locality helps.
* [ ] Move substantial unit suites to sibling `tests.rs` or `tests/` modules.
* [ ] Avoid enormous source files containing both implementation and hundreds of test lines.
* [ ] Preserve access to private implementation only where the tested invariant requires it.

#### Crate integration tests

Use `<crate>/tests/` for:

```text
public API contracts
cross-module workflows
serialization compatibility
canonical facade paths
consumer-visible behavior
```

* [ ] Use only public APIs.
* [x] Add facade integration tests that do not import `vvm-core`.
* [x] Add package-specific integration tests for `vvm-build` and `cargo-vvm`.
* [ ] Keep persisted schema and golden-output compatibility tests at integration level.

#### Compile-time tests

Use `trybuild` for:

```text
derive and attribute macros
compile-fail diagnostics
typestate method availability
generated-code paths
invalid API combinations
```

* [ ] Review every `.stderr` fixture manually.
* [ ] Avoid snapshots for diagnostics that are not intended to be stable.
* [ ] Preserve spans and actionable compiler messages.

#### Fixture workspaces

Use isolated Cargo projects for:

```text
vvm-build consumer builds
cargo-vvm orchestration
package installation
clean build scripts
Cargo metadata behavior
packaged-crate use
```

* [x] Prevent fixtures from joining the parent workspace accidentally.
* [x] Give nested Cargo executions isolated target directories.
* [x] Avoid requiring Verilator for pure orchestration fixtures.
* [x] Add dedicated Verilator fixtures where native integration is the subject.

#### End-to-end tests

Add release-gating external workflows:

* [x] Clean external project builds a generated DUT using `vvm-build`.
* [x] Counter deterministic and randomized tests run successfully.
* [x] A deliberate mismatch retains useful diagnostics.
* [x] Tracing generates a readable VCD.
* [x] Timing-enabled delayed events execute correctly.
* [x] Multi-clock execution preserves independent timing.
* [x] Inout resolution example exercises contention and floating policy.
* [x] `cargo vvm coverage` produces per-test artifacts, merged JSON, text, HTML, and CI metric.
* [x] A failing test still produces available coverage reports and retains its exit status.
* [x] A project built from packaged crate archives works without workspace paths.

### Existing-test migration

* [x] Inventory every existing test.
* [x] Classify it as unit, integration, compile-time, fixture, or end-to-end.
* [ ] Move misplaced tests without changing their coverage.
* [ ] Move example-like integration tests out of user-facing examples.
* [ ] Remove duplicate tests that prove the same behavior at several internal layers.
* [ ] Keep regression tests for every previously fixed bug.

### Coverage hardening

* [x] Produce line coverage per crate; branch output remains unavailable in the installed stable tool.
* [ ] Identify every completely untested production module.
* [ ] Add tests for every currently untested deterministic module.
* [x] Define explicit exclusions for generated code and narrowly justified FFI glue.
* [x] Establish a global coverage floor that cannot regress.
* [x] Prefer per-crate floors where one aggregate number hides weak crates.
* [x] Retain human-readable and Cobertura coverage artifacts in CI.
* [ ] Do not increase line coverage through meaningless assertion-free execution.

### Property-based testing

Evaluate property testing for:

* [x] Packed extraction and insertion round trips.
* [x] Signed conversion and sign-extension behavior.
* [x] Random replay determinism.
* [x] Scheduler ordering and time arithmetic.
* [ ] Coverage merge order independence.
* [ ] Artifact serialization round trips.
* [ ] Counter and ratio overflow invariants.

Use fixed seeds for reproducible failures.

### Mutation testing

* [x] Evaluate `cargo-mutants` for deterministic pure-Rust modules.
* [x] Establish an initial mutation baseline.
* [x] Run mutation testing manually or on a scheduled CI pipeline.
* [x] Do not make the entire mutation suite block every merge initially.
* [x] Investigate surviving mutations in safety-critical arithmetic and persistence logic.

### CI test matrix

Add jobs covering:

```text
minimum supported Rust
current stable Rust
minimum supported Verilator
current supported Verilator
pure-Rust tests without Verilator
native integration tests
package archive tests
documentation tests
```

* [x] Split fast pure-Rust tests from expensive native tests.
* [x] Preserve one contributor command equivalent to required CI checks.
* [x] Ensure test jobs do not depend on untracked generated files.

### Acceptance criteria

* [x] A written test-location policy exists.
* [x] Tests are consistently organized.
* [x] Public contracts are tested through integration tests.
* [x] Procedural macros retain compile-pass and compile-fail coverage.
* [x] Real end-to-end workflows run in CI.
* [x] No deterministic production module is completely untested without an explicit justification.
* [x] Coverage cannot regress silently.
* [x] Packaged consumer workflows are tested.

---

## 12.4 — Example curation and realistic showcases

### Objective

Make `examples/` a collection of understandable real verification use cases rather than a second integration-test directory.

### Example inventory

* [x] Inventory every current example.
* [x] Record the VVM or HDL feature it exists to exercise.
* [x] Identify examples that are primarily type-shape or code-generation fixtures.
* [x] Identify duplicate examples that teach no additional workflow.
* [x] Identify missing realistic use cases.

### Reclassification

Move implementation fixtures into dedicated test locations:

```text
tests/fixtures/hdl/
tests/fixtures/build/
tests/fixtures/generated/
```

Likely fixture-oriented cases include narrowly focused:

```text
packed arrays
packed enums
packed structs
wide transforms
unpacked arrays
single-purpose type mappings
```

* [x] Preserve their integration coverage after moving them.
* [x] Keep fixture names descriptive.
* [x] Do not expose fixture packages as recommended examples.
* [x] Keep fixtures buildable independently where useful.

### User-facing example ladder

#### Counter

Keep the counter as the beginner example.

It should demonstrate:

```text
DUT build
Drive and Sample
Clock
reference model
scoreboard
registered tests
random replay
tracing
functional coverage
cargo-vvm reporting
```

* [x] Keep it minimal enough for a first-time user.
* [x] Ensure its README follows the stabilized API.
* [x] Include expected commands and output.

#### Synchronous FIFO

Add or promote a real FIFO example demonstrating:

```text
structured transactions
ready/valid or push/pop flow
backpressure
queue reference model
underflow and overflow behavior
longer randomized sequences
functional coverage
```

* [x] Provide meaningful normal and error scenarios.
* [x] Include occupancy and operation crosses.
* [x] Include deterministic replay.

#### Protocol or timing example

Add a UART, timer, or similar real timed design demonstrating:

```text
timing-enabled execution
protocol reconstruction
error injection
nontrivial observations
functional coverage
```

* [x] Keep the RTL understandable.
* [x] Document timing assumptions.
* [x] Avoid turning the example into a full protocol verification framework.

#### Multi-clock example

Replace or expand the synthetic multi-clock case with an asynchronous FIFO or another understandable CDC design.

* [x] Demonstrate independent clocks and phase relationships.
* [x] Demonstrate deterministic same-time ordering.
* [x] Include meaningful verification goals.
* [x] Clearly state that the example is not a formal CDC proof.

#### Bus-peripheral example

Evaluate a small APB, Wishbone, or similarly compact peripheral.

* [ ] Demonstrate structured bus transactions.
* [ ] Demonstrate register-model-like reference behavior without introducing a premature register abstraction.
* [ ] Demonstrate protocol and data functional coverage.

### Example quality standard

Every user-facing example should include:

```text
README
design overview
block diagram
verification goals
features demonstrated
run commands
expected result
coverage workflow where relevant
known limitations
```

* [x] Keep example code idiomatic and reviewed as public documentation.
* [x] Avoid unexplained helper machinery.
* [x] Ensure examples use only public supported APIs.
* [x] Run every example in CI.
* [x] Keep example output deterministic where practical.

### Acceptance criteria

* [x] `examples/` contains only projects intended for users to study.
* [x] Narrow code-generation cases live under fixtures.
* [x] Counter remains a clear beginner workflow.
* [x] At least one realistic sequential design is documented.
* [x] At least one meaningful timing or multi-clock design is documented.
* [x] Every retained example has complete user-facing documentation.
* [x] Every example runs in CI.

---

## 12.5 — mdBook, rustdoc, and GitLab Pages

### Objective

Publish one coherent project book containing the user guide, developer guide, examples, architecture documentation, and links to generated API documentation.

### mdBook layout

Create:

```text
docs/book/
├── book.toml
└── src/
    ├── 404.md
    ├── SUMMARY.md
    ├── introduction.md
    ├── why-vvm.md
    ├── how-vvm-fits-into-hdl-verification.md
    ├── how-vvm-works.md
    ├── rust-essentials-for-hdl-engineers.md
    ├── installation.md
    ├── quick-start.md
    ├── concepts/
    ├── guide/
    ├── api-guide/
    ├── examples/
    └── development/
```

Closing hygiene outcome:

- Hidden compatibility source chapters were removed from `docs/book/src`.
- Only `docs/book/src/SUMMARY.md` and `docs/book/src/404.md` remain unindexed.
- Legacy public URLs are preserved by generated redirects in `scripts/doc-suite.sh`.
- The public coverage artifact schema is rendered at `development/coverage-json-schema-v1.md`.

### User guide

Document:

* [x] Project purpose and supported use cases.
* [x] Installation and prerequisites.
* [x] First VVM project.
* [x] `vvm-build` and `build.rs`.
* [x] Generated DUT inclusion.
* [x] Driving and sampling transactions.
* [x] Clock configuration.
* [x] Testbench construction.
* [x] Reference models and scoreboards.
* [x] Registered VVM tests.
* [x] Failure policies and diagnostics.
* [x] Randomization and replay.
* [x] Tracing.
* [x] Multi-clock execution.
* [x] Timing-enabled models.
* [x] Inout behavior and limitations.
* [x] Functional coverage.
* [x] `cargo vvm coverage`.
* [x] CI integration.
* [x] Troubleshooting.

### Developer guide

Document:

* [x] Workspace architecture.
* [x] Supported public crate boundaries.
* [x] Verilator invocation and metadata pipeline.
* [x] Generated C++ adapter.
* [x] CXX bridge generation.
* [x] Generated Rust wrapper.
* [x] DUT lifecycle.
* [x] FFI and unsafe invariants.
* [x] Testbench execution ordering.
* [x] Multi-clock scheduler.
* [x] Timing scheduler.
* [x] Randomization architecture.
* [x] Coverage architecture.
* [x] Procedural macro architecture.
* [x] Error and diagnostic policy.
* [x] Testing strategy.
* [x] Benchmark policy.
* [x] Compatibility policy.
* [x] Release process.

### Rustdoc integration

Build a Pages tree conceptually shaped as:

```text
public/
├── index.html
├── api/
│   ├── vvm/
│   ├── vvm_build/
│   ├── vvm_core/
│   └── vvm_macros/
└── coverage/
```

* [x] Build mdBook into the Pages root.
* [x] Build rustdoc for all published library crates.
* [x] Publish rustdoc below `/api/`.
* [x] Add API-reference landing pages in the book.
* [x] Preserve rustdoc search and source navigation.
* [x] Link the book and API documentation bidirectionally.
* [x] Decide whether `cargo-vvm` command documentation is book-only.

### Package READMEs

Create or thoroughly update:

```text
crates/vvm/README.md
crates/vvm-build/README.md
crates/cargo-vvm/README.md
```

* [x] Ensure each README works independently on crates.io.
* [x] Give each package a focused purpose statement.
* [x] Include the minimum useful installation example.
* [x] Link to the project book.
* [x] Link to the relevant API documentation.
* [x] Avoid using workspace-relative links that break on crates.io.
* [x] Add `readme` and `documentation` metadata to package manifests.

### Documentation quality gates

* [x] Enable or retain missing-public-doc warnings.
* [x] Run all rustdoc tests.
* [x] Check internal and external links.
* [x] Check mdBook builds with warnings treated seriously.
* [x] Check code examples against the stabilized API.
* [x] Remove stale alpha workflows and manual coverage instructions.
* [x] Document Linux-only support explicitly if that remains the supported platform.
* [x] Document supported Rust and Verilator versions.

### GitLab Pages

* [x] Add a Pages stage and job.
* [x] Build mdBook and rustdoc reproducibly.
* [x] Publish only from intended branches or tags.
* [x] Retain build artifacts for troubleshooting.
* [x] Verify relative links under the GitLab Pages base path.
* [x] Publish version or release information visibly.
* [x] Verify the deployed site from CI.

### Acceptance criteria

* [x] The project book builds locally and in CI.
* [x] User and developer guides are substantial and navigable.
* [x] API documentation is available below the same Pages site.
* [x] `vvm-build` and `cargo-vvm` have crates.io-ready READMEs.
* [x] Documentation examples compile.
* [x] GitLab Pages deployment succeeds.
* [x] Public package metadata points to valid documentation.

---

## 12.6 — Benchmark suite and performance baselines

### Objective

Create deterministic developer-facing Criterion benchmarks that reveal before-and-after performance changes in VVM's pure-Rust runtime, coverage system, generated integration, native execution paths, and orchestration workflows.

### Benchmark policy

* [x] Define what VVM performance claims and does not claim.
* [x] Keep Criterion as the only measurement and comparison framework.
* [x] Use fixed seeds and stable representative inputs.
* [x] Record hardware, OS, compiler, Rust, Cargo, C++ compiler, and Verilator versions.
* [x] Treat baselines as local and machine-specific under `target/criterion`.
* [x] Establish local baselines before setting any future regression thresholds.
* [x] Do not make ordinary merge requests fail on noisy performance changes.

### Criterion benchmarks

Add benchmark targets for:

* [x] Packed extraction and insertion at representative widths.
* [x] Signed packed conversion.
* [x] `Bits` and `SignedBits` operations.
* [x] Random value generation.
* [x] Replayable sequence generation.
* [x] Exact scoreboard comparisons.
* [x] Mock-DUT testbench cycle overhead.
* [x] Multi-clock scheduler event processing.
* [x] Timing scheduler event processing.
* [x] Coverpoint sampling.
* [x] Cross sampling.
* [x] Coverage snapshot capture.
* [x] Coverage artifact encoding.
* [x] Coverage artifact decoding.
* [x] Coverage merging at several sizes.
* [x] Text report generation.
* [x] HTML report generation.
* [x] Verilator metadata normalization.
* [x] Generated-code model construction where practical.
* [x] Counter cycles per second.
* [x] FIFO transactions per second.
* [x] Baseline testbench execution without tracing or coverage.
* [x] Tracing overhead.
* [x] Functional-coverage overhead.
* [x] Multi-clock overhead.
* [x] Timing-enabled overhead.
* [x] Clean `vvm-build` wall-clock time.
* [x] Incremental `vvm-build` wall-clock time.
* [x] `cargo-vvm` orchestration overhead.
* [x] Peak memory notes where practical without building a custom framework.

### Benchmark organization

Conceptual layout:

```text
crates/*/benches/
    pure-Rust and package-local Criterion targets

examples/*/benches/
    native Verilator-backed Criterion targets
```

* [x] Avoid requiring Verilator for pure-Rust Criterion benchmarks.
* [x] Keep native system benchmarks separately selectable.
* [x] Avoid benchmarking debug builds.
* [x] Avoid accidental tracing or logging in benchmark paths.
* [x] Verify benchmark inputs are not optimized away.

### Contributor workflow and CI

* [x] Add contributor commands for full-suite runs, saved baselines, baseline comparison, and focused targets.
* [x] Document the local before-and-after workflow.
* [x] Keep benchmark execution out of `just ci`, pre-commit, and GitLab CI.
* [x] Record one initial `v0.2.0` local profile with real environment details and representative measurements.
* [x] Investigate statistically meaningful regressions with Criterion's reports.
* [x] Defer hard automatic thresholds until local workflow stability is demonstrated.

### Acceptance criteria

* [x] Criterion benchmarks cover the main pure-Rust hot paths.
* [x] Representative real-DUT system benchmarks exist.
* [x] Benchmarks use deterministic inputs.
* [x] Benchmark commands are documented.
* [x] Benchmarks remain local-only and CI does not execute them.
* [x] A representative `v0.2.0` local performance profile exists.

---

## 12.7 — Packaging, compatibility, and release engineering

### Objective

Verify that every supported VVM package can be independently packaged, installed, documented, and consumed outside the workspace.

### Package publication policy

Confirm the publication set and order.

Proposed order:

```text
vvm-core
vvm-macros
vvm-build
vvm-rs
cargo-vvm
```

* [ ] Confirm whether every package name is available and appropriate.
* [ ] Remove `publish = false` from packages intended for release.
* [ ] Keep implementation-only crates publishable only where dependency resolution requires it.
* [ ] Document the supported direct-entry packages.

### Package metadata

For every published package, review:

```text
name
version
description
readme
documentation
homepage
repository
license
keywords
categories
rust-version
include / exclude
```

* [ ] Ensure descriptions are specific and useful.
* [ ] Ensure package READMEs render on crates.io.
* [ ] Ensure documentation links are public.
* [ ] Ensure license declarations match included files.
* [ ] Ensure package archives contain all generated templates, fixtures, and native sources required by consumers.
* [ ] Exclude development-only and oversized files.

### Package verification

Run for every published crate:

```console
cargo package --list
cargo package
cargo publish --dry-run
```

* [ ] Review every package file list manually.
* [ ] Test packages in publication order.
* [ ] Build consumers from generated `.crate` archives or a local registry.
* [ ] Ensure no package relies on workspace-only paths.
* [ ] Ensure `vvm-build` can locate all packaged templates and support files.
* [ ] Ensure `cargo-vvm` installs and runs from its package archive.
* [ ] Test documentation links from packaged READMEs.

### External consumer fixtures

Add clean external projects that use released package shapes:

* [ ] Minimal counter consumer.
* [ ] Coverage-enabled consumer.
* [ ] Timing-enabled consumer.
* [ ] Multi-clock consumer where supported.
* [ ] `cargo-vvm` installed-command workflow.

The fixtures must not rely on:

```text
workspace path dependencies
untracked generated files
repository-only environment
parent Cargo configuration
```

### Compatibility matrix

Define and document:

```text
minimum supported Rust version
current stable Rust
supported Verilator range
supported C++ compiler range
supported operating systems
timing-enabled C++ requirements
```

* [ ] Verify the declared MSRV.
* [ ] Add MSRV CI.
* [ ] Test current stable Rust.
* [ ] Select and test a minimum supported Verilator.
* [ ] Test a current supported Verilator.
* [ ] Test GCC and Clang where practical.
* [ ] State Linux-only support explicitly if other systems are not verified.
* [ ] Avoid implying support for untested platforms.

### Public compatibility policy

* [ ] Define the compatibility promise beginning with `v0.2.0`.
* [ ] Define how pre-`1.0` semver changes will be handled.
* [ ] Define compatibility expectations for persisted coverage schemas.
* [ ] Define compatibility expectations for generated-code layouts.
* [ ] Define compatibility expectations for CLI output used by CI.
* [ ] Add API-diff checks against the accepted release baseline.
* [ ] Add schema fixture checks for persisted formats.

### Release documents

Create:

```text
CHANGELOG.md
RELEASING.md
SECURITY.md
CONTRIBUTING.md
```

* [ ] Use a consistent changelog format.
* [ ] Describe the release checklist.
* [ ] Describe publication order.
* [ ] Describe tag and GitLab Release creation.
* [ ] Describe version updates.
* [ ] Describe release rollback or yanking policy.
* [ ] Describe security-reporting channels.
* [ ] Describe contributor validation requirements.

### Release automation

* [ ] Add a tag-triggered release pipeline.
* [ ] Validate versions against the tag.
* [ ] Run all release gates before publishing.
* [ ] Build and verify package archives.
* [ ] Publish crates in dependency order.
* [ ] Generate a GitLab Release.
* [ ] Deploy versioned documentation.
* [ ] Retain package and documentation artifacts.
* [ ] Avoid publishing from unreviewed branch pipelines.
* [ ] Document required protected variables and tokens.

### Acceptance criteria

* [ ] Every intended crate passes `cargo publish --dry-run`.
* [ ] Package archives work in clean external consumers.
* [ ] `cargo-vvm` is installable.
* [ ] The support matrix is explicit and tested.
* [ ] Changelog and release documentation exist.
* [ ] Tag release automation is implemented and tested without performing an accidental production publication.
* [ ] A release-candidate publication process is ready.

---

## 12.8 — Safety, dependency, and final release audit

### Objective

Perform the final technical and repository-level audit before freezing the release candidate.

### FFI and unsafe audit

Review:

* [ ] Every handwritten unsafe block.
* [ ] Every generated unsafe block.
* [ ] CXX bridge signatures.
* [ ] DUT ownership and pinning.
* [ ] Verilator context and model lifetimes.
* [ ] Finalization and drop ordering.
* [ ] Trace object lifetime.
* [ ] Timing-enabled coroutine integration.
* [ ] Panic behavior across FFI.
* [ ] C++ exception containment.
* [ ] Thread confinement.
* [ ] `Send` and `Sync` behavior.
* [ ] Aliasing and mutable-reference assumptions.
* [ ] Inout transfer and buffer-size invariants.

For every unsafe operation:

* [ ] Document the safety invariant.
* [ ] Verify generated code preserves the invariant.
* [ ] Add a regression test where practical.
* [ ] Remove unnecessary unsafe code.
* [ ] Ensure no Rust panic or C++ exception crosses an unsupported boundary.

### Dependency audit

* [ ] Run `cargo audit`.
* [ ] Run `cargo deny check`.
* [ ] Run `cargo udeps`.
* [ ] Review duplicate dependency versions.
* [ ] Review license compatibility.
* [ ] Reject unapproved Git dependencies in published packages.
* [ ] Review default features.
* [ ] Review proc-macro dependency footprint.
* [ ] Review CLI dependency footprint.
* [ ] Commit required deny policy configuration.
* [ ] Document advisory exceptions with expiration or review criteria.

### Reproducibility audit

* [ ] Verify generated source is deterministic.
* [ ] Verify metadata normalization is deterministic.
* [ ] Verify coverage artifacts and reports are deterministic.
* [ ] Verify package contents do not depend on untracked files.
* [ ] Verify clean builds work from fresh clones.
* [ ] Verify fixture workspaces are isolated.
* [ ] Verify documentation builds without network-only local assumptions.
* [ ] Verify release commands use locked dependencies where appropriate.

### Repository hygiene

* [ ] Remove stale implementation notes from user-facing docs.
* [ ] Remove obsolete examples and fixtures.
* [ ] Remove dead feature flags.
* [ ] Remove unused dependencies.
* [ ] Remove obsolete lint allowances.
* [ ] Remove temporary compatibility aliases not intended for release.
* [ ] Resolve or explicitly defer every release-related `TODO`.
* [ ] Review ignored tests.
* [ ] Review committed generated files.
* [ ] Review package sizes.
* [ ] Review repository links and badges.

### Release gate

Run the complete release-equivalent validation:

```text
formatting
Clippy
unit tests
integration tests
trybuild tests
end-to-end tests
doctests
coverage
package verification
dependency audit
documentation build
Pages build
benchmark smoke run
```

* [ ] Record exact tool versions.
* [ ] Retain all reports.
* [ ] Resolve every release-blocking failure.
* [ ] Document accepted non-blocking limitations.

### Acceptance criteria

* [ ] Unsafe and FFI invariants are documented and reviewed.
* [ ] Dependency and license policies pass.
* [ ] Clean and packaged builds are reproducible.
* [ ] No release-blocking stale code or documentation remains.
* [ ] The full release validation succeeds.
* [ ] Remaining limitations are documented.

---

## 12.9 — VVM v0.2.0 release candidate

### Objective

Publish and dogfood a release candidate using the exact package, documentation, and automation path intended for `v0.2.0`.

### Feature freeze

* [ ] Freeze public features.
* [ ] Permit only release-blocking fixes.
* [ ] Require explicit review for API changes.
* [ ] Update the version to `0.2.0-rc.1`.
* [ ] Update package dependency versions consistently.
* [ ] Update changelog and release notes.

### Release-candidate validation

* [ ] Run the complete release gate.
* [ ] Generate all package archives.
* [ ] Test package archives in external consumer projects.
* [ ] Install and run `cargo-vvm` from the release-candidate package.
* [ ] Build the full book and API documentation.
* [ ] Compare public API against the accepted pre-release baseline.
* [ ] Compare benchmark results with the stored baseline.
* [ ] Verify persisted coverage fixtures.
* [ ] Verify GitLab CI and Pages deployment.

### Publication

* [ ] Publish or otherwise distribute `0.2.0-rc.1` packages in dependency order.
* [ ] Create the release-candidate tag.
* [ ] Create a GitLab prerelease.
* [ ] Publish release-candidate documentation.
* [ ] Verify crates.io package pages and README rendering.
* [ ] Verify docs.rs builds.
* [ ] Verify `cargo install cargo-vvm --version 0.2.0-rc.1`.

### Dogfooding

Test the release candidate in projects that do not use repository path dependencies:

* [ ] Minimal counter project.
* [ ] Existing counter example copied outside the workspace.
* [ ] Coverage and reporting workflow.
* [ ] Timing-enabled example.
* [ ] Multi-clock example.
* [ ] At least one realistic FIFO or protocol example.
* [ ] Clean environment or container build.

### Release-candidate exit criteria

* [ ] No known release-blocking correctness issue remains.
* [ ] No accidental public export remains.
* [ ] Package installation and external consumption work.
* [ ] Documentation is deployed and accurate.
* [ ] CI and coverage workflows work with published packages.
* [ ] Benchmark results show no unexplained major regression.
* [ ] Any required RC fixes are documented in the changelog.
* [ ] A decision to publish `v0.2.0` is recorded.

---

## 12.10 — VVM v0.2.0 release

### Objective

Publish the first supported VVM release after the release candidate has completed external validation.

### Final preparation

* [ ] Apply only approved release-candidate fixes.
* [ ] Re-run the complete release gate.
* [ ] Update versions from `0.2.0-rc.1` to `0.2.0`.
* [ ] Update internal dependency requirements.
* [ ] Finalize `CHANGELOG.md`.
* [ ] Finalize release notes.
* [ ] Verify the release commit contains no unintended changes.
* [ ] Verify the public API diff from the final RC is intentional.

### Publication

Publish in dependency order:

```text
vvm-core
vvm-macros
vvm-build
vvm-rs
cargo-vvm
```

* [ ] Wait for each crate to become available before publishing dependents.
* [ ] Verify checksums and package contents.
* [ ] Create the `v0.2.0` tag.
* [ ] Create the GitLab Release.
* [ ] Publish final versioned documentation.
* [ ] Verify GitLab Pages.
* [ ] Verify docs.rs.
* [ ] Verify crates.io READMEs and metadata.
* [ ] Verify `cargo install cargo-vvm --version 0.2.0`.

### Post-publication verification

Using only published artifacts:

* [ ] Create a new counter project.
* [ ] Build a DUT through `vvm-build`.
* [ ] Run deterministic and randomized tests.
* [ ] Generate tracing output.
* [ ] Run `cargo vvm coverage`.
* [ ] Verify merged JSON, text, HTML, and GitLab metric.
* [ ] Verify one timing-enabled workflow.
* [ ] Verify one multi-clock workflow.
* [ ] Verify package documentation links.
* [ ] Verify the recorded MSRV.

### Baselines

* [ ] Store the final `v0.2.0` public API baseline.
* [ ] Store the final persisted-schema fixtures.
* [ ] Store the final generated-code fixtures.
* [ ] Store the final benchmark baseline.
* [ ] Record supported Rust, Verilator, compiler, and OS versions.
* [ ] Open post-`0.2.0` roadmap items separately from release issues.

### Release completion criteria

* [ ] All intended crates are published.
* [ ] `cargo-vvm` installs successfully.
* [ ] External projects work using only published packages.
* [ ] Documentation is deployed.
* [ ] The support and compatibility policy is public.
* [ ] No release-blocking issue is known.
* [ ] The repository roadmap marks Milestone 12 complete.
* [ ] Development resumes under a post-`0.2.0` roadmap.

---

## Milestone 12 cross-cutting acceptance criteria

Milestone 12 is complete when:

* [ ] The release scope is frozen and unfinished older work is explicitly classified.
* [ ] The facade uses coherent canonical modules.
* [ ] The flat root re-export surface is removed.
* [ ] The prelude is intentionally small.
* [ ] Public errors and diagnostics follow one documented architecture.
* [ ] Public error variants and sources are comprehensively tested.
* [ ] Tests follow a documented unit/integration/compile-time/end-to-end taxonomy.
* [ ] No deterministic production module is completely untested without justification.
* [ ] Real external workflows are tested.
* [ ] User-facing examples are realistic, documented designs.
* [ ] Narrow implementation fixtures no longer masquerade as examples.
* [ ] The mdBook user and developer guides are complete.
* [ ] Rustdoc and mdBook are deployed together through GitLab Pages.
* [ ] `vvm-build` and `cargo-vvm` have crates.io-ready READMEs.
* [ ] The benchmark suite produces a reviewed `v0.2.0` baseline.
* [ ] All intended packages pass package and dry-run publication checks.
* [ ] Published-package consumer fixtures work.
* [ ] The supported Rust, Verilator, compiler, and OS matrix is explicit.
* [ ] Release and security documentation exists.
* [ ] Unsafe and FFI invariants are reviewed and documented.
* [ ] Dependency, advisory, and license policies pass.
* [ ] `v0.2.0-rc.1` is published and externally validated.
* [ ] `v0.2.0` is published successfully.
* [ ] Final API, schema, generated-code, and benchmark baselines are retained.

## Milestone 12 non-goals

Do not use pre-release cleanup to introduce:

* a new verification component hierarchy;
* a UVM-style phase system;
* a new asynchronous runtime;
* arbitrary new HDL feature families;
* N-way coverage crosses;
* a register abstraction without a demonstrated example;
* broad platform claims without CI evidence;
* hard benchmark thresholds before runner stability is known;
* compatibility aliases that undermine the canonical API cleanup;
* a requirement to complete every aspirational Milestone 11 item before release.

The objective is a credible, coherent, tested, documented, and publishable first release—not an attempt to finish every possible VVM feature.

---

# 18. Testing strategy

## `vvm-build`

- [ ] Builder validation tests.
- [ ] Path normalization tests.
- [ ] Command construction tests.
- [ ] Cargo directive tests.
- [ ] Metadata normalization tests.
- [ ] Signal type mapping tests.
- [ ] Naming-rule tests.
- [ ] Unsupported-feature tests.
- [ ] Versioned Verilator JSON fixtures.
- [ ] Generated C++ snapshot tests.
- [ ] Generated CXX bridge snapshot tests.
- [ ] Generated Rust wrapper snapshot tests.
- [ ] Counter clean-build integration test.
- [ ] Counter incremental-build integration test.
- [ ] Native build failure diagnostic test.

## `vvm-core`

- [ ] Pure Rust mock DUT.
- [ ] Drive semantics.
- [ ] Sample semantics.
- [ ] Stateful reference model.
- [ ] Exact scoreboard.
- [ ] Stop-on-first-failure.
- [ ] Collect-all-failures.
- [ ] Finalization after success.
- [ ] Finalization after error.
- [ ] Deterministic cycle ordering.

## `vvm-macros`

- [ ] Attribute parser unit tests.
- [ ] Expansion unit tests.
- [ ] `trybuild` pass tests.
- [ ] `trybuild` compile-fail tests.
- [ ] Generic type tests.
- [ ] Span-quality checks where feasible.

## Examples

- [ ] Counter.
- [ ] Combinational adder.
- [ ] FIFO.
- [ ] Randomized sequential model.
- [ ] Tracing.
- [ ] Multiple independently discoverable Rust tests.

---

# 19. Documentation plan

## During development

- [ ] Keep this plan updated.
- [ ] Add architecture decision records for major choices.
- [ ] Document supported Verilator versions.
- [ ] Document supported port types.
- [ ] Document simulation ordering.
- [ ] Document generated file layout.
- [ ] Document build prerequisites.

## MVP documentation

- [ ] README overview.
- [ ] Installation prerequisites.
- [ ] Counter quickstart.
- [ ] `build.rs` guide.
- [ ] Transactions and derives.
- [ ] Reference models.
- [ ] Scoreboards.
- [ ] Standard Rust test integration.
- [ ] Tracing.
- [ ] Environment-based test configuration.
- [ ] Native-build troubleshooting.
- [ ] Unsupported HDL constructs.
- [ ] FFI and safety explanation.

---

# 20. Risks and mitigation

## Generated CXX bridge under `OUT_DIR`

**Risk:** The generated Rust bridge must be processed by both `cxx-build` and Rust compilation.

- [ ] Prove this early in Milestone 4.
- [ ] Add clean-build and incremental-build tests.
- [ ] Keep paths deterministic.

## Verilator JSON stability

**Risk:** Verilator's JSON format may change.

- [ ] Isolate raw parsing.
- [ ] Normalize immediately.
- [ ] Store versioned fixtures.
- [ ] Define a supported version range.
- [ ] Fail clearly on unknown layouts.

## Native linking complexity

**Risk:** Model objects, Verilator runtime, CXX shims, and the C++ standard library must link correctly.

- [ ] Keep the first spike Linux-only if necessary.
- [ ] Capture complete diagnostics.
- [ ] Avoid abstracting before the build works.
- [ ] Add platform support incrementally.

## Procedural macro complexity

**Risk:** Macro ergonomics may obscure compiler errors.

- [ ] Implement traits manually first.
- [ ] Keep derive syntax narrow.
- [ ] Use `trybuild`.
- [ ] Preserve spans.
- [ ] Let Rust type checking validate port direction and types.

## Premature UVM imitation

**Risk:** Recreating UVM concepts could make VVM-rs clunky again.

- [ ] Require a demonstrated Rust use case for every abstraction.
- [ ] Prefer ownership and iterators over component graphs.
- [ ] Avoid phases, factories, and registries until needed.
- [ ] Regularly review example verbosity.

## Strict lint configuration

**Risk:** Native integration and generated code may be made unnecessarily painful by global lint policies.

- [ ] Keep strict lints in handwritten Rust.
- [ ] Add narrow allowances in generated modules.
- [ ] Avoid weakening workspace standards globally.
- [ ] Do not let lint cleanup indefinitely block integration spikes.

---

# 21. Decision log

| ID | Decision | Status |
|---|---|---|
| D-001 | Implement VVM-rs as a Rust-first framework around Verilator-generated C++ models. | Accepted |
| D-002 | Generate a C++ adapter instead of binding directly to Verilator headers. | Accepted |
| D-003 | Use CXX for the Rust/C++ bridge. | Accepted |
| D-004 | Keep `vvm-core` independent of Verilator and CXX. | Accepted |
| D-005 | Use `Iterator<Item = Stimulus>` before defining a sequence trait. | Accepted |
| D-006 | Handwrite the first bridge before implementing bridge generation. | Accepted |
| D-007 | Handwrite `Drive` and `Sample` before implementing derives. | Accepted |
| D-008 | Remove `vvm-ffi` until shared ABI functionality is concretely needed. | Accepted |
| D-009 | Keep the public facade nearly empty until stable APIs exist. | Accepted |
| D-010 | Start with unsigned top-level ports no wider than 64 bits. | Proposed |
| D-011 | Start with a synchronous cycle-based runner and no async runtime. | Proposed |
| D-012 | Use Verilator JSON for port metadata extraction. | Proposed |
| D-013 | Use PIMPL for generated C++ adapter headers. | Proposed |
| D-014 | Make `finish()` idempotent and call it from the adapter destructor. | Proposed |

---

# 22. MVP definition of done

## Build integration

- [ ] A user declares a DUT in `build.rs`.
- [ ] Verilator is invoked automatically.
- [ ] Ports are discovered automatically.
- [ ] C++ adapter code is generated.
- [ ] CXX bridge code is generated.
- [ ] Native code is compiled and linked automatically.
- [ ] Unsupported HDL ports fail with useful errors.

## Safe DUT API

- [ ] The user receives a safe Rust DUT wrapper.
- [ ] Input ports have typed setters.
- [ ] Output ports have typed getters.
- [ ] `eval()` is safe to call.
- [ ] DUT finalization is reliable.
- [ ] CXX details do not leak into ordinary user code.

## Verification core

- [ ] Stimuli implement or derive `Drive`.
- [ ] Observations implement or derive `Sample`.
- [ ] Sequences can be ordinary iterators.
- [ ] Stateful reference models work.
- [ ] Scoreboards produce structured failures.
- [ ] A reusable synchronous testbench runner exists.

## User experience

- [ ] Counter works from a clean clone.
- [ ] A deliberate mismatch produces useful diagnostics.
- [ ] Public rustdoc contains an end-to-end example.
- [ ] Build prerequisites are documented.
- [ ] The API is materially simpler than the original C++ VVM.

---

# 23. Immediate next steps

Complete these in exact order:

- [ ] Remove placeholder APIs and tests from workspace crates.
- [ ] Add `cxx` and `cxx-build` dependencies.
- [ ] Create the `examples/counter` crate.
- [ ] Add `rtl/counter.sv`.
- [ ] Verify the counter with direct Verilator linting.
- [ ] Invoke Verilator manually from the example's `build.rs`.
- [ ] Write the PIMPL C++ adapter.
- [ ] Write the CXX bridge.
- [ ] Compile and link the generated model.
- [ ] Construct the DUT from Rust.
- [ ] Toggle reset and clock.
- [ ] Read and verify the counter output.
- [ ] Confirm correct finalization.
- [ ] Only then extract build logic into `vvm-build`.

---

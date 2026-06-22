# VVM-rs Implementation Plan

> **Project:** VVM-rs — Verilator Verification Methodology in Rust
> **Status:** Early bootstrap
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

Status: **parked until a genuinely shared FFI abstraction appears**.

Potential future responsibilities:

- Fixed-width bit-vector transfer types.
- Common C++ support headers.
- Shared bridge errors.
- Tracing configuration types.
- Internal ABI utilities.

DUT-specific bridge code must remain generated in the consuming crate.

---

## 4. Milestone status

| Milestone | Name | Status |
|---:|---|---|
| 0 | Workspace preparation | Complete |
| 1 | Handwritten Rust ↔ CXX ↔ Verilator bridge | Complete |
| 2 | Reusable Verilator build orchestration | Not started |
| 3 | Verilator metadata extraction | Not started |
| 4 | Generated DUT bridge | Not started |
| 5 | Safe generated DUT API | Not started |
| 6 | Minimal pure-Rust VVM core | Not started |
| 7 | Testbench runner | Not started |
| 8 | Derive macros | Not started |
| 9 | Public facade | Not started |
| 10 | Tracing, reporting, and CLI | Not started |
| 11 | Wider HDL feature support | Backlog |

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
- [x] Create the `vvm-ffi` crate.
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

- [x] `cargo run -p vvm-example-counter` succeeds.
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

- [ ] Add `Testbench::new`.
- [ ] Require a DUT at construction.
- [ ] Add sequence configuration.
- [ ] Add reference-model configuration.
- [ ] Add scoreboard configuration.
- [ ] Add clock-driving configuration.
- [ ] Use type-state only if it improves errors materially.
- [ ] Avoid boxed trait objects initially.
- [ ] Preserve concrete generic types.

## Run loop

- [ ] Iterate over stimuli.
- [ ] Drive each stimulus.
- [ ] Perform the defined clock sequence.
- [ ] Evaluate at documented points.
- [ ] Sample outputs.
- [ ] Obtain expected outputs.
- [ ] Invoke the scoreboard.
- [ ] Track cycles and successful checks.
- [ ] Track failures.
- [ ] Stop or continue according to configuration.
- [ ] Always finish the DUT.

## Results

Suggested shape:

```rust
pub struct TestResult<F> {
    pub cycles: u64,
    pub checks: u64,
    pub failures: Vec<F>,
}
```

- [ ] Preserve cycle numbers.
- [ ] Preserve expected and observed values.
- [ ] Optionally preserve stimuli.
- [ ] Add compact and detailed reporting.
- [ ] Define example process-exit behavior.

## Failure policy

- [ ] Support stop-on-first-failure.
- [ ] Support collecting multiple failures.
- [ ] Allow a maximum failure count.
- [ ] Prevent unbounded diagnostic storage.
- [ ] Distinguish simulation errors from check failures.
- [ ] Ensure finalization after errors.

## Acceptance criteria

- [ ] Counter contains no handwritten simulation loop.
- [ ] Successful checks are reported.
- [ ] Intentional failures include cycle-aware diagnostics.
- [ ] The runner is tested with a pure Rust mock DUT.
- [ ] No procedural macros are used yet.

---

# 13. Milestone 8 — Derive macros

## Goal

Generate repetitive `Drive` and `Sample` implementations after their handwritten forms are proven.

## Intended API

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
└── sample/
    ├── mod.rs
    ├── input.rs
    └── expand.rs
```

## Shared attribute parsing

- [ ] Parse `#[vvm(dut = path)]`.
- [ ] Parse `#[vvm(port)]`.
- [ ] Parse `#[vvm(port = "hdl_name")]`.
- [ ] Reject duplicate DUT attributes.
- [ ] Reject duplicate port attributes.
- [ ] Reject unknown options.
- [ ] Preserve useful spans.
- [ ] Avoid panics for malformed input.

## `Drive` derive

- [ ] Accept named-field structs.
- [ ] Reject tuple structs and enums.
- [ ] Require a DUT path.
- [ ] Generate one setter call per mapped field.
- [ ] Use field names by default.
- [ ] Support explicit HDL names.
- [ ] Preserve generics and where clauses.
- [ ] Use hygienic VVM trait paths.
- [ ] Produce readable expanded code.

## `Sample` derive

- [ ] Accept named-field structs.
- [ ] Reject tuple structs and enums.
- [ ] Require a DUT path.
- [ ] Generate one getter call per mapped field.
- [ ] Construct observations with named fields.
- [ ] Support explicit HDL names.
- [ ] Preserve generics and where clauses.
- [ ] Produce readable expanded code.

## Direction and type checking

Rely initially on generated DUT method availability:

- Driving an output calls a nonexistent setter.
- Sampling an input calls a nonexistent getter.
- Incompatible field types fail ordinary Rust type checking.

- [ ] Confirm resulting diagnostics are understandable.
- [ ] Add custom macro diagnostics only where they help.
- [ ] Avoid duplicating DUT metadata in macro input initially.

## `trybuild` tests

Pass cases:

- [ ] Basic `Drive`.
- [ ] Basic `Sample`.
- [ ] Explicit port rename.
- [ ] Generic input where valid.
- [ ] Coexistence with unrelated derives.

Fail cases:

- [ ] Missing DUT attribute.
- [ ] Enum derives `Drive`.
- [ ] Tuple struct derives `Sample`.
- [ ] Unknown `vvm` option.
- [ ] Duplicate port metadata.
- [ ] Invalid port syntax.
- [ ] Driving an output.
- [ ] Sampling an input.
- [ ] Incompatible field type.
- [ ] Missing setter or getter.

## Acceptance criteria

- [ ] Handwritten counter `Drive` implementation is removed.
- [ ] Handwritten counter `Sample` implementation is removed.
- [ ] Runtime behavior is unchanged.
- [ ] Compile-fail tests verify diagnostics.
- [ ] Macro entry points remain thin.
- [ ] Macro internals are tested through `proc_macro2`.

---

# 14. Milestone 9 — Public facade

## Goal

Expose only stable, proven APIs.

Possible contents:

```rust
pub use vvm_core::{
    Drive,
    Dut,
    ReferenceModel,
    Sample,
    Scoreboard,
    TestResult,
    Testbench,
};

pub use vvm_macros::{
    Drive,
    Sample,
};
```

## Re-export checklist

- [ ] Re-export stable core traits.
- [ ] Re-export stable result types.
- [ ] Re-export stable derive macros.
- [ ] Add facade-level crate documentation.
- [ ] Add a minimal prelude.
- [ ] Verify derive and trait name coexistence.
- [ ] Avoid re-exporting internal parser or codegen types.
- [ ] Keep `vvm-build` as a direct build dependency.
- [ ] Decide whether `vvm-ffi` remains private.

## DUT inclusion convenience

Potential API:

```rust
vvm::include_dut!(counter);
```

- [ ] Decide whether inclusion belongs in `vvm` or `vvm-build`.
- [ ] Define generated file naming conventions.
- [ ] Support multiple DUTs in one crate.
- [ ] Prevent module-name collisions.
- [ ] Keep generated implementation modules private by default.
- [ ] Document manual `include!` fallback.

## Acceptance criteria

- [ ] Example user code imports from `vvm`.
- [ ] Example `build.rs` imports from `vvm-build`.
- [ ] Ordinary users need no internal crates directly.
- [ ] The facade exposes no unstable implementation details.
- [ ] Public rustdoc shows an end-to-end counter example.

---

# 15. Milestone 10 — Tracing, reporting, and CLI

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
- [ ] Support listing tests.
- [ ] Support selecting one test.
- [ ] Support a default test.
- [ ] Reject duplicate names.
- [ ] Decide whether registration uses a procedural macro.
- [ ] Keep registration independent of the run loop.

## CLI

Target:

```text
cargo run -- \
    --test counter_random \
    --seed 1234 \
    --cycles 10000 \
    --trace counter.fst
```

- [ ] Select a CLI parsing crate.
- [ ] Add `--list`.
- [ ] Add `--test`.
- [ ] Add `--seed`.
- [ ] Add `--cycles`.
- [ ] Add `--trace`.
- [ ] Add stop-on-failure controls.
- [ ] Define useful exit codes.
- [ ] Keep CLI support optional.

## Acceptance criteria

- [ ] A test can be selected by name.
- [ ] A deterministic seed is displayed and replayable.
- [ ] A waveform is generated.
- [ ] Failure reports are reproducible.
- [ ] Library users can opt out of the CLI layer.

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

- [ ] Represent named clocks.
- [ ] Define independent periods.
- [ ] Define same-time ordering.
- [ ] Add scheduler support.
- [ ] Add a multi-clock example.

## Inout ports

- [ ] Determine Verilator representation.
- [ ] Separate value, output enable, and sampled input.
- [ ] Design a safe Rust API.
- [ ] Add a tri-state example.
- [ ] Document two-state limitations.

## Arrays and aggregates

- [ ] Parse unpacked dimensions.
- [ ] Design array accessors.
- [ ] Parse packed structs and unions.
- [ ] Decide flattened versus typed representations.
- [ ] Verify bit layouts carefully.
- [ ] Add integration tests.

## Timing-enabled models

- [ ] Add a distinct timing scheduler.
- [ ] Support `eventsPending()`.
- [ ] Support `nextTimeSlot()`.
- [ ] Define time advancement.
- [ ] Integrate waveform dumping.
- [ ] Add a delay-based example.
- [ ] Preserve cycle-based mode.

## DPI and coverage

- [ ] Identify inbound and outbound DPI use cases.
- [ ] Define safe callback ownership.
- [ ] Define panic behavior across FFI.
- [ ] Enable Verilator coverage generation.
- [ ] Write coverage data on finish.
- [ ] Document merge and reporting tools.

---

# 17. Testing strategy

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
- [ ] Multiple-test registry.

---

# 18. Documentation plan

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
- [ ] Testbench runner.
- [ ] Tracing.
- [ ] Native-build troubleshooting.
- [ ] Unsupported HDL constructs.
- [ ] FFI and safety explanation.

---

# 19. Risks and mitigation

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

# 20. Decision log

| ID | Decision | Status |
|---|---|---|
| D-001 | Implement VVM-rs as a Rust-first framework around Verilator-generated C++ models. | Accepted |
| D-002 | Generate a C++ adapter instead of binding directly to Verilator headers. | Accepted |
| D-003 | Use CXX for the Rust/C++ bridge. | Accepted |
| D-004 | Keep `vvm-core` independent of Verilator and CXX. | Accepted |
| D-005 | Use `Iterator<Item = Stimulus>` before defining a sequence trait. | Accepted |
| D-006 | Handwrite the first bridge before implementing bridge generation. | Accepted |
| D-007 | Handwrite `Drive` and `Sample` before implementing derives. | Accepted |
| D-008 | Park `vvm-ffi` until shared ABI functionality is identified. | Accepted |
| D-009 | Keep the public facade nearly empty until stable APIs exist. | Accepted |
| D-010 | Start with unsigned top-level ports no wider than 64 bits. | Proposed |
| D-011 | Start with a synchronous cycle-based runner and no async runtime. | Proposed |
| D-012 | Use Verilator JSON for port metadata extraction. | Proposed |
| D-013 | Use PIMPL for generated C++ adapter headers. | Proposed |
| D-014 | Make `finish()` idempotent and call it from the adapter destructor. | Proposed |

---

# 21. MVP definition of done

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

# 22. Immediate next steps

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

## Progress notes

Use this section for dated notes.

### YYYY-MM-DD

- **Work completed:**
- **Problems encountered:**
- **Decisions made:**
- **Next action:**

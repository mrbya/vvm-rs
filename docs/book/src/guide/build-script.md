# Build Script

`build.rs` is the point where Cargo asks VVM to generate the DUT wrapper.

## Why This Exists

Verilator generation must happen before your Rust test code compiles. `build.rs`
is Cargo's standard hook for that preparation step.

## Minimal Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/build.rs:build-script}}
```

## What `DutBuilder` Owns

[`DutBuilder`](../api/vvm_build/struct.DutBuilder.html) owns the declarative
build configuration for one generated wrapper.

In the minimal example above it defines:

- the logical wrapper name;
- the top HDL module;
- the HDL source file list;
- whether a trace-capable wrapper should be generated;
- the actual generation step.

## Common Options

- `.top_module("...")` selects the HDL module to elaborate.
- `.source("...")` adds a source file.
- `.trace(...)` enables VCD-capable wrapper generation.
- timing-related options enable the wrapper features needed by timed DUTs.
- Verilator-selection options let you override the executable when required.

When a chapter or CI environment needs the exact methods, use the
[DutBuilder API Guide](../api-guide/dut-builder.md) and the rustdoc for
[`DutBuilder`](../api/vvm_build/struct.DutBuilder.html).

## Variations

For more than one source file, add more `.source(...)` calls.

For include-directory or define-heavy builds, keep `build.rs` declarative: use it
to describe the DUT build, not to embed testbench logic.

For timed or traced models, remember that those options change generated-wrapper
capabilities, not just the command line passed to Verilator.

## Build Errors

Build-stage failures usually come from one of four places:

- Verilator is missing or the wrong executable is selected.
- The HDL source list or top-module name is wrong.
- The DUT uses unsupported or not-yet-mapped port shapes.
- The logical DUT name later disagrees with `include_dut!`.

When the build fails, start with the `build.rs` inputs before debugging your
Rust testbench code.

## Common Mistakes

- Hiding the logical DUT name behind unnecessary indirection.
- Doing filesystem or testbench work in `build.rs` that belongs in normal Rust
  code.
- Forgetting that build-script changes can affect regeneration.

## Related Material

- [Project Setup](project-setup.md)
- [Including The DUT](including-the-dut.md)
- [DutBuilder API Guide](../api-guide/dut-builder.md)

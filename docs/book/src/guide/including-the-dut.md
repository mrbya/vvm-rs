# Including The DUT

After `build.rs` generates the wrapper, your Rust code brings it into scope with
`vvm::include_dut!`.

## Minimal Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:include-dut}}
```

## What It Does

[`vvm::include_dut!`](../api/vvm/macro.include_dut.html) expands to an `include!`
of generated Rust code stored under Cargo's `OUT_DIR`.

The macro argument must match the logical DUT name from `DutBuilder::new(...)`.

## Where To Invoke It

Most users invoke it in `src/lib.rs` behind `#[cfg(test)]` because the generated
wrapper is only needed while compiling tests.

If your non-test code also needs the wrapper, you can include it in a broader
scope, but keep the dependency placement in `Cargo.toml` consistent with that
choice.

## Generated Naming

For a logical name such as `event_counter`, the generated module is usually named
after that logical identifier and exposes a generated DUT type such as
`EventCounter` plus its generated error type.

## Debugging Name Mismatches

If inclusion fails, check these three names in order:

1. the logical name in `DutBuilder::new("...")`;
2. the `include_dut!(...)` argument;
3. the generated type you use in derives and tests.

## Multiple Generated DUTs

If one crate generates more than one wrapper, invoke `include_dut!` for each
logical name and keep the resulting modules clearly separated in the code.

## Trace And Timing Variants

Trace-capable and timing-capable methods exist only when the wrapper was built
with the corresponding generation options. If a method seems to be missing, look
back at `build.rs` before assuming the runtime API is broken.

## Inspecting Generated Output

When debugging generation issues, inspect Cargo's build output under `target/`
and `OUT_DIR` instead of editing the generated files. The generated code is a
diagnostic artifact, not a long-term source file.

## Related Material

- [Build Script](build-script.md)
- [Generated Types Reference](../reference/generated-types.md)
- [DutBuilder API Guide](../api-guide/dut-builder.md)

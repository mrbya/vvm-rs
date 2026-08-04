# Project Setup

Most VVM verification projects are Rust library crates.

That layout works well because it gives Cargo one place to run `build.rs`, one
place to compile generated DUT glue, and one place to discover ordinary Rust
tests.

## Why This Exists

Before you can write stimulus, models, or scoreboards, you need a crate layout
that lets Cargo run Verilator at build time and compile the generated wrapper at
test time.

## Recommended Layout

```text
my-verification-crate/
├── Cargo.toml
├── build.rs
├── rtl/
│   └── event_counter.sv
└── src/
    └── lib.rs
```

Use this shape unless you already have a strong reason to split the work across
multiple crates.

## Dependency Roles

The minimal dependency setup looks like this:

```toml
{{#include ../../../../tests/fixtures/docs-quick-start/Cargo.toml:dependencies}}
```

- `[dev-dependencies]` are available to tests. `vvm-rs` usually belongs here
  because most users only need the VVM facade while compiling tests.
- `[build-dependencies]` are available to `build.rs`. `vvm-build` belongs here
  because Cargo uses it before the crate itself compiles.
- `[dependencies]` is for code needed by the normal library or binary build. You
  often do not need `vvm-rs` there unless your non-test build also uses the
  generated wrapper.

## Where HDL And Generated Code Live

- Put HDL under a directory such as `rtl/`.
- Keep `build.rs` at the package root.
- Include generated wrappers from Rust source with `vvm::include_dut!(...)`.
- Let Cargo keep generated files under `OUT_DIR` and other target directories.

Generated files are build artifacts. Do not commit them.

## Logical DUT Name

The logical DUT name is the string you pass to `DutBuilder::new("...")`.

That name must match the later `vvm::include_dut!(...)` invocation. It is how the
build-time generation step and the Rust-side inclusion step refer to the same
wrapper.

## One Crate Or Several?

One crate is often enough when:

- one team owns the DUT and its tests together;
- one or a few related DUTs share the same verification support code;
- you want the simplest Cargo workflow.

A separate verification crate is often better when:

- the DUT source already lives in another package or repository;
- you want to keep verification-only dependencies out of the application crate;
- multiple verification harnesses need to target the same design differently.

## Multiple DUTs In One Crate

One crate can generate more than one DUT wrapper by invoking `DutBuilder` more
than once in `build.rs` and including each logical name from Rust. Keep the
naming explicit so each wrapper has an obvious owner.

## Cargo Rebuild Tracking

At a high level, Cargo reruns `build.rs` when relevant inputs change. VVM and
`build.rs` use Cargo rebuild directives so HDL-source changes and important build
configuration changes cause regeneration when needed.

## Common Mistakes

- Putting `vvm-build` under `[dependencies]` instead of `[build-dependencies]`.
- Forgetting `build = "build.rs"` in the package section.
- Checking generated wrapper files into source control.
- Mismatching the logical DUT name and `include_dut!` argument.
- Treating the verification crate like an HDL source tree instead of a Cargo
  project with HDL inputs.

## Related Material

- [Build Script](build-script.md)
- [Including The DUT](including-the-dut.md)
- [DutBuilder API Guide](../api-guide/dut-builder.md)

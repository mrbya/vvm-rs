# Architecture

The workspace is intentionally split into clear boundaries.

- `vvm-rs` / `vvm` is the public facade.
- `vvm-core` owns runtime primitives and coverage internals.
- `vvm-build` owns Verilator invocation and generated bridge creation.
- `vvm-macros` owns derives and `#[vvm::test]`.
- `cargo-vvm` is a direct-entry binary for offline coverage workflows.
- curated examples and fixtures are consumers, not public API crates.

The direction is outward from build and core internals toward the facade and the
consumer crates. Application code should land on `vvm`, not on `vvm-core`.

# Project Setup

The common VVM project shape has four pieces:

- `Cargo.toml` with `vvm-rs` in `[dev-dependencies]` and `vvm-build` in
  `[build-dependencies]`;
- `build.rs` that runs `DutBuilder`;
- HDL sources such as `rtl/*.sv`;
- Rust test code that includes the generated DUT wrapper.

This layout keeps the generated wrapper private to the crate while still letting
tests use the public `vvm` facade.

Keep the logical DUT name stable. That name is the thread connecting
`DutBuilder::new("...")` to `vvm::include_dut!(...)`.

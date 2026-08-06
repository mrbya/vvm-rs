# vvm-core

`vvm-core` contains VVM's reusable pure-Rust verification primitives.

Most application code should depend on the `vvm-rs` facade crate as `vvm` rather
than importing `vvm-core` directly.

## What Lives Here

- DUT and testbench traits
- cycle-driven and timing-aware runtime primitives
- packed-value helpers
- functional coverage data models, persistence, merge, and reporting support
- deterministic randomization support used by the public facade and tooling

## When To Use It Directly

Use `vvm-core` directly only when you are building orchestration or integration
layers that intentionally sit below the public facade, such as package tooling.

## Documentation

- Facade quick start: <https://byacrates.gitlab.io/vvm-rs/quick-start.html>
- Compatibility guide: <https://byacrates.gitlab.io/vvm-rs/guide/compatibility-and-limitations.html>
- API reference: <https://byacrates.gitlab.io/vvm-rs/api/vvm_core/>

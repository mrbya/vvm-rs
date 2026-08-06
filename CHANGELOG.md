# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic Versioning].

## [Unreleased]

- Upcoming release line: `v0.2.0`

## [0.1.0-alpha.1] - 2026-07-03

### Added

- Initial alpha release of the VVM Rust workspace.
- `vvm-rs` facade crate for Rust-first verification of Verilator-generated RTL models.
- `vvm-core` verification primitives for typed testbench execution, timing, failures, outcomes, and reporting.
- `vvm-build` support for Verilator invocation, metadata extraction, generated DUT bridge code, and native build integration from `build.rs`.
- `vvm-macros` derives for typed DUT driving, sampling, and clock control.
- `#[vvm::test]` for defining VVM tests as ordinary Rust tests.
- Standard `cargo test` and `cargo nextest run` execution for VVM tests.
- Environment-based test overrides with `VVM_SEED`, `VVM_REPLAY`, `VVM_CYCLES`, and `VVM_TRACE_DIR`.
- Deterministic randomization with replay tokens for reproducible randomized verification.
- Reference models, exact-equality scoreboards, configurable failure policies, and structured mismatch reporting.
- Explicit simulation timing and cycle-aware testbench execution.
- VCD waveform tracing for trace-capable tests.
- Working `examples/counter` example crate covering generated DUT usage, deterministic smoke testing, randomized regression, and failure-reporting behavior.
- Repository verification workflow through `just`, GitLab CI, and the containerized development image used by CI.

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
[Unreleased]: https://gitlab.com/byacrates/vvm-rs/-/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://gitlab.com/byacrates/vvm-rs/-/tags/v0.1.0-alpha.1

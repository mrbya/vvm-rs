# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic Versioning].

## [Unreleased]

Upcoming release line: `v0.2.0`

### Added

- A complete mdBook-based user guide, API guide, development guide, and examples path covering project setup, timing-enabled models, bidirectional ports, functional coverage, troubleshooting, and maintained example workflows.
- New maintained example crates for synchronous FIFO, timed UART, asynchronous FIFO, and tri-state bus verification alongside the original counter example.
- Functional-coverage persistence, deterministic merge/report support, and the publishable `cargo-vvm` Cargo subcommand for suite-level coverage workflows.
- Native generated-port regression fixtures covering signed values, wide ports, arrays, packed structs and enums, timing-enabled DUTs, and multi-clock DUTs.
- Packaged-consumer and packaged-installation validation proving released crate shapes work outside the workspace.
- An explicit compatibility matrix and accepted `v0.2.0` API baseline for the release line.

### Changed

- The public facade was reorganized around canonical modules instead of the earlier flatter export surface, and the reviewed `v0.2.0` API is now treated as the supported baseline for `0.2.x` patch releases.
- Native support claims are now explicitly Linux-only for the `v0.2.0` cycle, with Rust 1.87.0 plus current stable Rust, Verilator 5.000 minimum, Verilator 5.050 current validation, GCC-based native CI, and Linux Clang native-fixture validation documented as the tested matrix.
- Timing-enabled generated builds now document the actual native requirement: ordinary DUTs use C++17 while timing-enabled DUTs require a C++20 compiler with coroutine support.
- Contributor and maintainer documentation now covers Backlog workflow, release-facing validation, package verification, dependency and license policy through `cargo-deny`, reproducibility checks through `just repro-check`, the `just release-audit` gate, and public API review against the checked-in `v0.2.0` baseline.

### Fixed

- Package metadata, included support files, and extracted crate layouts now hold up under `cargo package`, `cargo publish --dry-run`, and packaged consumer validation instead of depending on workspace-only assumptions.
- Public coverage schema documentation now matches the implemented deterministic merge behavior and fingerprint-based compatibility checks.
- Generated DUT wrappers now enforce a conservative thread-confinement policy instead of leaving `Send` and `Sync` behavior implicit at the Rust/C++ boundary.
- Release-facing Cargo and documentation commands now use locked dependency resolution where appropriate, and the assembled documentation build metadata no longer varies on the current wall clock for one commit.

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

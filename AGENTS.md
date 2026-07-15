# VVM Agent Guide

## Workspace

- Rust 2024 workspace (MSRV 1.87) for Verilator-backed verification; building the example crates and native integration requires Verilator plus a C++ toolchain.
- `crates/vvm` is the public facade (`vvm-rs` package, `vvm` library); `vvm-core` owns runtime/testbench primitives, `vvm-build` runs Verilator and generates/compiles bridges from consumer `build.rs`, and `vvm-macros` owns derives and `#[vvm::test]`.
- `examples/counter` is the reference end-to-end DUT integration. Other `examples/*` exercise RTL port shapes and are workspace members.
- Generated DUT source is included from `OUT_DIR`; keep `DutBuilder::new("name")` and `vvm::include_dut!(name)` aligned.

## Commands

- Prefer `just` recipes.
- Run `just init` once to install contributor tooling, including nightly Rust, nextest, coverage, udeps, audit, Markdown TOC, and pre-commit.
- Format with `just fmt` or check without edits using `just fmt --check`; formatting explicitly uses `cargo +nightly fmt --all`.
- Run linting with `just check -- -D warnings`; it checks all workspace targets, tests, examples, and features.
- Run the normal full suite with `just test`; it invokes `cargo nextest run --all-features --workspace`. For focused runs, use Cargo/nextest package filtering, e.g. `cargo nextest run -p vvm-build` or `cargo test -p vvm-example-counter <test-name>`.
- `just ci` is the CI-equivalent, non-mutating verification: format check, lint with warnings denied, `cargo +nightly udeps`, audit, doctests, and coverage. It writes coverage reports under `coverage/`.
- The pre-commit hook runs `just ci` for Rust/TOML/justfile changes. README changes also run `just index`, which rewrites the README TOC.

## Tests And Fixtures

- VVM tests use normal Rust discovery. Unit-style VVM tests must be in `#[cfg(test)]`; integration tests under `tests/` are already test-only.
- The `vvm-macros` compile-test suite uses `trybuild`; failure diagnostics in `crates/vvm-macros/tests/ui/fail/*.stderr` are expected snapshots and must be updated deliberately with macro diagnostic changes.
- `vvm-build` codegen and Verilator metadata fixtures under `crates/vvm-build/tests/fixtures/` are checked-in golden inputs/outputs. The metadata fixtures are versioned for Verilator 5.048; do not casually regenerate or reformat them.
- Replayable VVM tests can be reproduced with `VVM_REPLAY` (takes precedence over `VVM_SEED`); `VVM_CYCLES` and `VVM_TRACE_DIR` configure supported tests, with default trace output rooted at `target/vvm-trace/`.

## Rustdoc And Lints
- The root workspace enables strict `missing_docs` and `clippy::missing_docs_in_private_items` plus other strict code readability and bug-proning lints.
- Do not add new `#[allow(...)]` attributes just to silence clippy; fix the warning/error instead.
- When adding or rewriting docs, match `docs/dev/rustdoc_style.md` rather than the stale README link to `dev/docs/rustdoc_style.md`.

# AGENTS

## Workspace Shape
- This is a Cargo workspace only at the root; the main library crates are `crates/vvm`, `crates/vvm-core`, `crates/vvm-build`, and `crates/vvm-macros`, plus the example package at `examples/counter`.
- `crates/vvm` is the public runtime facade package. Its Cargo package name is `vvm-rs`, but its library name is `vvm`; use `-p vvm-rs` when filtering Cargo commands.
- Runtime crates should depend on `vvm`; `build.rs` should depend on `vvm-build` directly.
- Generated DUT code is included from `OUT_DIR`; `vvm` provides `include_dut!` and hidden `__private::cxx` support so user crates do not need a direct runtime `cxx` dependency.
- `docs/dev/reference-projects/vvm` is a git submodule that holds the legacy/reference project, not the active workspace code.

## Tooling And Commands
- Follow `justfile` recipes instead of guessing raw commands.
- Bootstrap once with `just init`; it installs nightly, `cargo-nextest`, `cargo-llvm-cov`, `cargo-udeps`, `cargo-audit`, `markdown-toc`, and `pre-commit`.
- Formatting is nightly-only here: `just fmt` runs `cargo +nightly fmt --all`.
- Linting: `just check -- -D warnings` matches the repo's strict clippy workflow.
- Tests: `just test` runs `cargo nextest run --all-features --workspace`.
- Doc tests: `just doctest`.
- Full CI-equivalent verification: `just ci`.
- Workspace `cargo check`, `cargo test`, and clippy now build `examples/counter`, whose `build.rs` runs `verilator --lint-only` and compiles a tiny CXX bridge; those commands require `verilator` and a working C++ toolchain.
- The interactive pre-push/pre-commit sweep is heavier than CI: `just pre-commit` runs format, strict checks, doctests, and coverage.

## Focused Verification
- For one package, prefer Cargo package filters, for example `cargo test -p vvm-rs`.
- The facade integration test target is `cargo test -p vvm-rs --test public_api`.
- The repository integration test target is `cargo test -p vvm-rs --test integration_tests`.
- The counter example smoke target is `cargo run -p vvm-example-counter`.
- For HDL-only verification without Cargo, use `verilator --lint-only examples/counter/rtl/counter.sv`.

## Hooks And Generated Changes
- `justfile` has `set dotenv-load := true`, so `just` recipes automatically load `.env`.
- Local pre-commit hooks run `just ci` for changes touching `.rs`, `.toml`, or `justfile`.
- README edits trigger `just index`, which rewrites `README.md` with `markdown-toc -i`.
- Commit messages are checked by `pre-commit-conventional-commits`; keep commit messages in conventional-commit form.
- Coverage output from CI-style runs is written to `coverage/cobertura.xml` and `coverage/coverage.txt`.

## Style Constraints
- Workspace `rust-version` is `1.85.0` and edition is `2024`.
- `rustfmt.toml` enforces `imports_granularity = "Module"` and `group_imports = "StdExternalCrate"`; do not treat import ordering as default rustfmt behavior.

## Rustdoc And Lints
- The workspace crates enable strict `missing_docs` and `clippy::missing_docs_in_private_items` and other crate-level lints.
- Do not add new `#[allow(...)]` attributes just to silence clippy; fix the warning unless an existing local test pattern clearly applies.
- When adding or rewriting docs, match `docs/dev/rustdoc_style.md`.

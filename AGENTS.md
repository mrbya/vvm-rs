# AGENTS

## Workspace
- Root-only Cargo workspace members are `crates/vvm`, `crates/vvm-core`, `crates/vvm-build`, `crates/vvm-macros`, and `examples/counter`.
- `crates/vvm` is the public facade crate, but its Cargo package name is `vvm-rs`; use `-p vvm-rs` for package-scoped Cargo commands.
- Runtime code should depend on `vvm`; `build.rs` should depend on `vvm-build`.
- Generated DUT modules come from `OUT_DIR`; consumers should use `vvm::include_dut!`, and `vvm` already re-exports the hidden `cxx` support generated code needs.
- `docs/dev/reference-projects/vvm` is a git submodule with legacy/reference material, not the active workspace implementation.

## Commands
- Prefer `just` recipes over ad hoc Cargo commands.
- Bootstrap once with `just init`.
- Format with `just fmt`; this repo uses `cargo +nightly fmt --all`.
- Lint with `just check -- -D warnings`.
- Run tests with `just test`; run doc tests with `just doctest`.
- Run CI-equivalent verification with `just ci`. GitLab CI uses this directly from `.gitlab-ci.yml`; there are no repo GitHub workflows.
- `just pre-commit` is heavier than CI: it formats, runs strict checks, doc tests, and coverage.

## Verification Shortcuts
- Facade integration target: `cargo test -p vvm-rs --test public_api`.
- Repository integration target: `cargo test -p vvm-rs --test integration_tests`.
- Counter example smoke test: `cargo run -p vvm-example-counter`.
- HDL-only lint for the example DUT: `verilator --lint-only examples/counter/rtl/counter.sv`.

## Toolchain Gotchas
- `examples/counter` is a workspace member, so `cargo check`, `cargo test`, and clippy build its `build.rs` too.
- That build script runs Verilator and compiles a small CXX bridge, so Rust verification commands need `verilator` plus a working C++ toolchain.
- `just` automatically loads `.env` because `justfile` sets `dotenv-load := true`.

## Hooks And Style
- Pre-commit runs `just ci` when `.rs`, `.toml`, or `justfile` changes.
- README edits trigger `just index`, which rewrites `README.md` with `markdown-toc -i`.
- Commit messages are checked for conventional-commit format.
- Workspace `edition` is `2024` and `rust-version` is `1.87.0`.
- `rustfmt.toml` enforces `imports_granularity = "Module"` and `group_imports = "StdExternalCrate"`.
- Workspace lints warn on `missing_docs` and `clippy::missing_docs_in_private_items`; follow `docs/dev/rustdoc_style.md` when adding or rewriting docs.

# AGENTS

## Workspace Shape
- This is a Cargo workspace only at the root; the member crates are `crates/vvm`, `crates/vvm-core`, `crates/vvm-build`, `crates/vvm-ffi`, and `crates/vvm-macros`.
- `crates/vvm` is the public facade package. Its Cargo package name is `vvm-rs`, but its library name is `vvm`; use `-p vvm-rs` when filtering Cargo commands.
- Current code is still scaffold-level: each crate only exposes `src/lib.rs` crate attributes/docs, and the only checked-in integration test and benchmark live under `crates/vvm/`.
- `docs/dev/reference-projects/vvm` is a git submodule that holds the legacy/reference project, not the active workspace code.

## Tooling And Commands
- Follow `justfile` recipes instead of guessing raw commands.
- Bootstrap once with `just init`; it installs nightly, `cargo-nextest`, `cargo-llvm-cov`, `cargo-udeps`, `cargo-audit`, `markdown-toc`, and `pre-commit`.
- Formatting is nightly-only here: `just fmt` runs `cargo +nightly fmt --all`.
- Linting: `just check -- -D warnings` matches the repo's strict clippy workflow.
- Tests: `just test` runs `cargo nextest run --all-features --workspace`.
- Doc tests: `just doctest`.
- Full CI-equivalent verification: `just ci`.
- The interactive pre-push/pre-commit sweep is heavier than CI: `just pre-commit` runs format, strict checks, doctests, and coverage.

## Focused Verification
- For one package, prefer Cargo package filters, for example `cargo test -p vvm-rs`.
- The only checked-in integration test target is `cargo test -p vvm-rs --test integration_tests`.
- Benchmarks are only defined for `crates/vvm`; use `just benchmark` or `just benchmark-target benches`.

## Hooks And Generated Changes
- `justfile` has `set dotenv-load := true`, so `just` recipes automatically load `.env`.
- Local pre-commit hooks run `just ci` for changes touching `.rs`, `.toml`, or `justfile`.
- README edits trigger `just index`, which rewrites `README.md` with `markdown-toc -i`.
- Commit messages are checked by `pre-commit-conventional-commits`; keep commit messages in conventional-commit form.
- Coverage output from CI-style runs is written to `coverage/cobertura.xml` and `coverage/coverage.txt`.

## Style Constraints
- Workspace `rust-version` is `1.85.0` and edition is `2024`.
- `rustfmt.toml` enforces `imports_granularity = "Module"` and `group_imports = "StdExternalCrate"`; do not treat import ordering as default rustfmt behavior.

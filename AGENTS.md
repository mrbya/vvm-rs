# AGENTS

## Architecture
- The workspace contains runtime crates (`vvm-core`, `vvm`, `vvm-macros`), build-time bridge generation (`vvm-build`), and executable examples under `examples/`.
- `crates/vvm` exposes Rust library `vvm` but its Cargo package is `vvm-rs`; use `-p vvm-rs` for package-scoped commands.
- Depend on `vvm` at runtime and `vvm-build` only from a consumer `build.rs`. `vvm-build` invokes Verilator, generates Rust/C++ bridge code, and compiles native support.
- Generated DUT modules live under `OUT_DIR`; include them with `vvm::include_dut!`. The facade re-exports the hidden `cxx` dependency that generated code needs.
- `docs/dev/reference-projects/vvm` is legacy submodule material, not active implementation code.

## Commands
- Prefer `just` recipes. `just fmt` requires nightly; `just check -- -D warnings` runs Clippy across workspace targets/examples; `just test` uses nextest with all features.
- `just ci` is the GitLab CI command: strict format/lint/udeps/audit, doc tests, then coverage. `just pre-commit` also reformats and runs local coverage.
- `just` loads `.env` automatically.

## Verification Shortcuts
- Facade integration: `cargo test -p vvm-rs --test public_api`; repository integration: `cargo test -p vvm-rs --test integration_tests`.
- Example smoke tests are package-scoped, e.g. `cargo test -p vvm-example-counter counter_smoke` or `cargo test -p vvm-example-unpacked-array`.
- HDL-only checks bypass Rust generation, e.g. `verilator --lint-only examples/counter/rtl/counter.sv`.

## Toolchain Gotchas
- Workspace examples have `build.rs` scripts; workspace Cargo commands require Verilator and a working C++ toolchain, not only Rust.
- Verilator metadata fixtures under `crates/vvm-build/tests/fixtures/verilator/5.048/` are intentionally excluded from automatic JSON reformatting; regenerate them with Verilator 5.048 when changing normalization/codegen inputs.

## Hooks And Style
- The pre-commit hook runs `just ci` for `.rs`, `.toml`, or `justfile` changes; README changes run `just index`. Commit messages must be conventional commits.
- The workspace is edition 2024 with Rust 1.87. `rustfmt.toml` needs nightly for module-granularity imports and grouped standard/external imports.
- `missing_docs` and private-item rustdoc lints are enabled. For hand-written Rust documentation, follow `docs/dev/rustdoc_style.md`; generated code is excluded.

## Rustdoc And Lints
- The root workspace enables strict `missing_docs` and `clippy::missing_docs_in_private_items` plus other strict code readability and bug-proning lints.
- Do not add new `#[allow(...)]` attributes just to silence clippy; fix the warning/error instead.
- When adding or rewriting docs, match `docs/dev/rustdoc_style.md` rather than the stale README link to `dev/docs/rustdoc_style.md`.


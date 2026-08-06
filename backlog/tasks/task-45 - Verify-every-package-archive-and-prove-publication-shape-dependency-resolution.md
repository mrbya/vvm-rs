---
id: TASK-45
title: Verify every package archive and prove publication-shape dependency resolution
status: Done
assignee:
  - OpenCode
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 15:16'
labels:
  - release
  - packaging
  - tests
milestone: m-5
dependencies:
  - TASK-44
  - TASK-51
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Exercise cargo package, cargo package --list, and cargo publish --dry-run for every intended published package, then extend the package-consumer infrastructure into a release-shape simulation that resolves only from packaged crate contents in the real publication order. Scope includes manual file-list review, staged dependency resolution without workspace fallbacks, isolated targets, and proving that packaged crates do not require workspace inheritance after packaging.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Each intended published crate passes cargo package --list, cargo package, and cargo publish --dry-run
- [x] #2 Package file lists are reviewed and required support files are present without leaking workspace-only paths or irrelevant files
- [x] #3 Publication-shape simulation resolves vvm-core, vvm-macros, vvm-build, vvm-rs, and cargo-vvm from packaged crate contents in dependency order
- [x] #4 Isolated temporary directories and isolated CARGO_TARGET_DIR values are used for package-consumer validation
- [x] #5 Failure diagnostics make missing packaged resources or dependency mismatches obvious
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review the existing packaged-consumer fixture and refactor its package extraction helpers so they can package all five publishable crates in dependency order with clearer archive-review diagnostics.
2. Extend the fixture coverage to prove publication-shape dependency resolution from extracted package contents for `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`, and `cargo-vvm`, using isolated temporary directories and isolated `CARGO_TARGET_DIR` values.
3. Run `cargo package --list`, `cargo package`, and `cargo publish --dry-run` for each publishable crate in dependency order, inspect the file lists, and fix any packaging or manifest-rewrite problems revealed by the dry runs.
4. Record package archive size and content findings, update any packaging configuration needed for missing support files or unintended inclusions, and verify there are no workspace-path leaks.
5. Re-run the focused package and fixture validation until every publishable crate and the publication-shape simulation pass cleanly.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Extended `crates/vvm/tests/fixtures.rs` with a new offline publication-shape resolution test that packages all five publishable crates, asserts their packaged manifests are self-contained, and checks each extracted package against only other extracted package archives.

Resolved a real publishability cycle by moving the trybuild macro UI harness from `vvm-macros` to `vvm-rs`, removing the publish-time `vvm-macros -> vvm-rs` dev-dependency loop while keeping the UI coverage on the public facade surface.

Used a clean `/tmp/opencode` snapshot of the current workspace plus temporary Cargo patch configuration to run `cargo package` and `cargo publish --dry-run` without `--allow-dirty`, while the extracted-archive fixture proves later packages do not rely on the source workspace tree.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Verified every publishable package archive and proved publication-shape dependency resolution for the `v0.2.0` line.

What changed:
- Added `packaged_publishable_crates_resolve_from_extracted_archives` to `crates/vvm/tests/fixtures.rs`. The test packages `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`, and `cargo-vvm`, asserts their packaged manifests are self-contained, and runs `cargo check --offline` on each extracted package using only patch entries that point at other extracted package archives.
- Added clearer packaged-manifest checks for workspace inheritance leakage and missing license files.
- Moved the trybuild UI harness from `crates/vvm-macros/tests/trybuild.rs` into `crates/vvm/tests/macro_ui.rs`, added `trybuild` as a `vvm-rs` dev-dependency, updated `justfile` and `.gitlab-ci.yml`, and refreshed the `.stderr` snapshots. This removed the publish-time `vvm-macros` dev-dependency cycle on `vvm-rs`.
- Established a clean-snapshot packaging workflow under `/tmp/opencode` so `cargo package` and `cargo publish --dry-run` can be executed without `--allow-dirty` while still validating the current uncommitted tree.

Files affected:
- `crates/vvm/tests/fixtures.rs`
- `crates/vvm/tests/macro_ui.rs`
- `crates/vvm/Cargo.toml`
- `crates/vvm-macros/Cargo.toml`
- `crates/vvm-macros/tests/ui/fail/*.stderr`
- `justfile`
- `.gitlab-ci.yml`
- removed `crates/vvm-macros/tests/trybuild.rs`

Package archive review results:
- `vvm-core`: archive includes README, dual-license files, benches, integration test, and the full `src/coverage` tree required by the public coverage modules.
- `vvm-macros`: archive includes README, dual-license files, macro sources, and the compile-fail snapshots used for diagnostics verification.
- `vvm-build`: archive includes README, dual-license files, codegen fixtures, and Verilator metadata fixtures needed for deterministic build/codegen verification.
- `vvm-rs`: archive includes README, dual-license files, public modules, and integration tests including fixture and macro UI coverage.
- `cargo-vvm`: archive includes README, dual-license files, orchestration sources, CLI tests, and the benchmark support module.
- No packaged manifest retained `workspace = true` inheritance or leaked absolute source-workspace paths.

Commands executed:
- `cargo test -p vvm-rs --test fixtures packaged_publishable_crates_resolve_from_extracted_archives -- --exact`
- `TRYBUILD=overwrite cargo test -p vvm-rs --test macro_ui -- --exact derive_ui`
- `cargo test -p vvm-rs --test macro_ui -- --exact derive_ui`
- `cargo package --list -p vvm-core`
- `cargo package --list -p vvm-macros`
- `cargo package --list -p vvm-build`
- `cargo package --list -p vvm-rs`
- `cargo package --list -p cargo-vvm`
- in a clean `/tmp/opencode` snapshot with internal patch configuration:
  `cargo package -p vvm-core`
  `cargo package -p vvm-macros`
  `cargo package -p vvm-build`
  `cargo package -p vvm-rs`
  `cargo package -p cargo-vvm`
  `cargo publish --dry-run -p vvm-core`
  `cargo publish --dry-run -p vvm-macros`
  `cargo publish --dry-run -p vvm-build`
  `cargo publish --dry-run -p vvm-rs`
  `cargo publish --dry-run -p cargo-vvm`

Validation results:
- All five package lists were reviewed.
- All five packages passed `cargo package` in the clean snapshot tree.
- All five packages passed `cargo publish --dry-run` in the clean snapshot tree.
- The extracted-archive simulation proves the package dependency graph resolves offline from packaged contents rather than the source workspace.
- Package verification no longer depends on the old `vvm-macros` publish-time dev-dependency cycle.

Recorded package sizes from `cargo package`:
- `vvm-core`: 685.2 KiB packaged, 134.7 KiB compressed
- `vvm-macros`: 142.7 KiB packaged, 28.3 KiB compressed
- `vvm-build`: 831.2 KiB packaged, 127.3 KiB compressed
- `vvm-rs`: 104.3 KiB packaged, 27.1 KiB compressed
- `cargo-vvm`: 102.7 KiB packaged, 26.7 KiB compressed

Release implications:
- Publication-order validation now has both direct Cargo dry-runs and an extracted-package resolution proof.
- The workspace is structurally ready for later release tooling to automate the same clean-snapshot packaging flow.

Retained limitations:
- This task proves package archives and package-level resolution. The separate external-consumer workflow fixtures and isolated `cargo-vvm` installation checks remain in later milestone tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

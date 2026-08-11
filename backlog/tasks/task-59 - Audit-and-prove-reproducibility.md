---
id: TASK-59
title: Audit and prove reproducibility
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:58'
updated_date: '2026-08-11 12:27'
labels: []
milestone: m-6
dependencies:
  - TASK-53
documentation:
  - scripts/release-verify.sh
  - docs/dev/implementation-plan.md
  - crates/vvm-build/tests/fixtures/
  - crates/vvm-core/tests/fixtures/coverage/
  - tests/fixtures/
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Prove that repository-controlled generated code, metadata normalization, coverage artifacts and reports, package contents, isolated consumers, and documentation builds are deterministic or intentionally normalized with explicit reasoning. Verify release-facing commands use locked dependency resolution where appropriate and strengthen automation where practical.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Generated source is deterministic
- [x] #2 Metadata normalization is deterministic
- [x] #3 Coverage artifacts and reports are deterministic
- [x] #4 Package contents depend only on tracked inputs
- [x] #5 Fresh isolated builds and consumers succeed
- [x] #6 Documentation builds reproducibly from tracked sources
- [x] #7 Release commands use locked dependencies where appropriate
- [x] #8 Reproducibility checks are automated where practical
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current reproducibility-sensitive surfaces: generated code snapshots, metadata fixtures, coverage artifacts and reports, package consumer fixtures, documentation builds, and release scripts.
2. Identify command-level nondeterminism risks such as missing `--locked`, local-state sensitivity, or output paths that leak machine-specific data.
3. Add the smallest reproducibility automation and script changes needed to make the repository checks explicit and repeatable.
4. Run focused reproducibility validation, record exact normalization assumptions, and summarize the remaining accepted limits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audited reproducibility-sensitive paths in codegen tests, coverage-report tests, isolated consumer fixtures, package-shape fixtures, documentation assembly, and release verification scripts.

Locked release-facing Cargo resolution in `scripts/release-verify.sh` with a `cargo metadata --locked` preflight plus `--locked` on package and extracted-package check commands.

Locked `cargo doc` in `scripts/doc-suite.sh` and replaced the doc-suite `built_at` wall-clock timestamp with the current commit timestamp so repeated builds of the same commit no longer vary on wall-clock time alone.

Added `just repro-check` to automate deterministic codegen, deterministic coverage-report rendering, isolated clean-consumer execution, packaged-crate validation, and documentation builds in one focused reproducibility gate.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the reproducibility audit and added focused automation for the release-facing deterministic surfaces.

What changed:
- Added `just repro-check` as a focused reproducibility gate.
- Updated `scripts/release-verify.sh` to enforce a locked lockfile preflight and to use `--locked` for `cargo package` and extracted-package `cargo check` steps.
- Updated `scripts/doc-suite.sh` so `cargo doc` runs with `--locked` and `public/build-info.json` uses the current commit timestamp instead of the current wall-clock time.

Reproducibility findings:
- Generated wrapper outputs were already covered by a deterministic codegen test and remained stable.
- Coverage text and metric rendering were already covered by a deterministic report test and remained stable.
- Package contents and isolated consumers were already validated by the fixture suite; this task added a focused way to rerun them as one reproducibility check.
- Documentation builds from tracked sources succeeded locally; the one avoidable nondeterministic field in the assembled site was the wall-clock `built_at` timestamp, which is now stable per commit.

Commands executed:
- `just repro-check` (rerun with a larger timeout after the packaged-fixture segment exceeded the initial session timeout, not due to a repository failure)
- `just docs-suite`
- script and fixture readback checks for locked-resolution coverage and deterministic fixture intent

Validation results:
- `cargo test -p vvm-build --lib generates_deterministic_output` passed.
- `cargo test -p vvm-core --lib renderers_and_metric_are_deterministic` passed.
- `cargo test -p vvm-rs --test fixtures clean_consumer_generates_and_executes_a_dut -- --exact` passed.
- `just test-package` passed inside `just repro-check`.
- `cargo test --workspace --doc` and `mdbook build docs/book` passed inside `just repro-check`.
- `just docs-suite` passed after the doc-suite reproducibility changes.

Normalization and accepted limits:
- Generated code and coverage-report checks require no normalization beyond the repository’s existing deterministic tests.
- Documentation assembly still records intentional provenance in `public/build-info.json`; `channel` and `revision` remain descriptive metadata, while `built_at` is now stable for one commit rather than depending on the current clock.
- Coverage and release-artifact provenance fields remain intentionally descriptive rather than stripped, which is acceptable so long as their variability is explicit and does not affect functional compatibility.
<!-- SECTION:FINAL_SUMMARY:END -->

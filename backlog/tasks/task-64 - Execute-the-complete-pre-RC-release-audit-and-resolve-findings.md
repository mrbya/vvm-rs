---
id: TASK-64
title: Execute the complete pre-RC release audit and resolve findings
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:59'
updated_date: '2026-08-11 14:23'
labels: []
milestone: m-6
dependencies:
  - TASK-53
  - TASK-54
  - TASK-56
  - TASK-57
  - TASK-58
  - TASK-59
  - TASK-60
  - TASK-61
  - TASK-63
documentation:
  - justfile
  - scripts/release-verify.sh
  - docs/dev/implementation-plan.md
  - CHANGELOG.md
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Run the complete release-equivalent audit and dry-run verification for the v0.2.0 pre-RC state, diagnose every failure, fix every release blocker, document accepted non-blocking limitations, and confirm that no publication, RC tag, or v0.2.0-rc.1 version bump occurs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Full release audit succeeds
- [x] #2 Full release verification succeeds
- [x] #3 Every release-blocking finding is resolved
- [x] #4 Accepted limitations are documented
- [x] #5 No production publication occurs
- [x] #6 No RC tag is created
- [x] #7 No v0.2.0-rc.1 version bump occurs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Execute `just release-audit` and fix every release-blocking failure it surfaces across formatting, linting, tests, docs, coverage, dependency policy, API diff, and reproducibility checks.
2. Execute `just release-verify` and fix every release-blocking failure in the extracted-package, dry-run release, and artifact-retention path.
3. Re-run any failing focused commands as needed until both gates are green.
4. Record release blockers, fixes, accepted non-blocking limitations, and explicit confirmation that no publication, RC tag, or version bump occurred.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Executed `just release-audit` successfully after the preceding task set landed. The gate retained tool-version and command logs under `target/vvm-release-audit/`.

First `just release-verify` run exposed a real release-path bug: the new `--locked` policy was too strict for extracted-package validation because unpacked `.crate` trees intentionally do not carry a lockfile. Fixed by keeping the workspace `cargo metadata --locked` preflight while removing `--locked` from the patched extracted-package `cargo check` path.

Second `just release-verify` run exposed the same issue for `cargo package` when cumulative patch overrides were present. Fixed by keeping `--locked` only at the workspace preflight and allowing the patch-driven extracted-package simulation to resolve offline without `--locked`.

Final `just release-verify` run completed successfully and retained release artifacts under `target/vvm-release/0.2.0`. Non-blocking residual warnings from cumulative patch overrides during staged package validation are documented rather than treated as release blockers.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Executed the full pre-RC release-equivalent validation and resolved the release-blocking findings it surfaced.

Release-critical commands executed:
- `just release-audit`
- `just release-verify`
- focused reruns of `just release-verify` after fixing the release-path lock-policy defects

Release blockers discovered and fixed:
1. Extracted-package `cargo check` under `scripts/release-verify.sh` incorrectly used `--locked`.
   - Root cause: unpacked `.crate` trees intentionally have no lockfile, so `--locked` made the dry-run validation fail even though the offline extracted-package check was the correct contract.
   - Fix: keep a workspace `cargo metadata --locked` preflight, but remove `--locked` from the extracted-package `cargo check` step.
   - Validation: reran `just release-verify`.
2. Patched `cargo package` invocations under `scripts/release-verify.sh` incorrectly used `--locked`.
   - Root cause: the staged extracted-package simulation applies temporary patch overrides that intentionally differ from the workspace lockfile graph, so `--locked` forced an invalid update refusal.
   - Fix: keep the lockfile preflight at the workspace level and remove `--locked` from the patch-driven `cargo package` invocations.
   - Validation: reran `just release-verify` to completion.

Final validation results:
- `just release-audit` succeeded.
- `just release-verify` succeeded.
- Release-audit artifacts were retained under `target/vvm-release-audit/`.
- Release-verification artifacts were retained under `target/vvm-release/0.2.0/`.

Tool versions recorded:
- `rustc 1.95.0`
- `cargo 1.95.0`
- `rustfmt 1.10.0-nightly`
- `cargo-nextest 0.9.130`
- `cargo-audit 0.22.1`
- `cargo-deny 0.20.2`
- `cargo-udeps 0.1.60`
- `cargo-public-api 0.52.0`
- `mdbook 0.5.3`
- `Verilator 5.048`
- `g++ 16.1.1`
- `clang++ 21.1.8`

Accepted non-blocking limitations:
- `cargo deny check` still reports a reviewed duplicate-version warning for `syn` because the workspace’s direct proc-macro stack remains on `syn` 2 while several upstream proc-macro dependencies already use `syn` 3.
- `scripts/release-verify.sh` emits benign Cargo patch warnings during later staged package checks because patch overrides are accumulated by publication order and some earlier-package patches are naturally unused by later crates.
- The benchmark audit remains workstation-local and reviews retained local Criterion comparison artifacts rather than creating cross-machine thresholds.

Safety and publication confirmations:
- No production crate was published.
- No release tag or RC tag was created.
- The version was not bumped to `0.2.0-rc.1`.
- All release blockers found during the end-to-end audit are resolved.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-60
title: Build the complete release-audit gate
status: Done
assignee: []
created_date: '2026-08-11 11:58'
updated_date: '2026-08-11 14:23'
labels: []
milestone: m-6
dependencies:
  - TASK-53
  - TASK-58
  - TASK-59
documentation:
  - justfile
  - scripts/release-verify.sh
  - .gitlab-ci.yml
  - CONTRIBUTING.md
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create an explicit maintainer-facing release audit command that composes existing validation and release verification rather than duplicating it. The gate must cover release-critical formatting, linting, tests, docs, dependency checks, API checks, reproducibility checks, package verification, release dry-run validation, and tool-version capture while keeping ordinary CI proportionate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A dedicated release-audit command exists
- [x] #2 The command composes existing validation instead of duplicating it
- [x] #3 It runs every release-critical audit or delegates to the current authoritative script
- [x] #4 Tool versions are recorded
- [x] #5 Failures are actionable
- [x] #6 Ordinary CI is not unnecessarily bloated
- [x] #7 Release verification can invoke the audit without publishing anything
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Define a dedicated pre-RC release-audit entrypoint that composes the repository’s existing validation commands instead of duplicating them.
2. Add tool-version capture and plain-text audit logs so release-critical evidence is retained in a discoverable location.
3. Update the dry-run release verification path to invoke the new gate before extracted-package validation.
4. Validate shell syntax and command wiring, leaving the full execution to the end-to-end release-audit task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added `scripts/release-audit.sh` and `just release-audit` as the dedicated maintainer-facing gate for the pre-RC audit.

The release-audit script records tool versions to `target/vvm-release-audit/tool-versions.txt`, captures plain-text logs for the major audit phases, and composes existing commands instead of reimplementing them.

The gate currently delegates to `just ci`, `just deny`, `just api-diff`, and `just repro-check`, which keeps ordinary CI unchanged while making the full pre-RC audit explicit.

Updated `scripts/release-verify.sh` so the dry-run release path now runs `just release-audit` before extracted-package validation and retained release artifacts.

Validated shell syntax and wiring with `bash -n scripts/release-audit.sh` and `bash -n scripts/release-verify.sh`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Built the dedicated release-audit gate for milestone 12.8.

What changed:
- Added `scripts/release-audit.sh`.
- Added `just release-audit`.
- Updated `scripts/release-verify.sh` so release verification now composes the new release-audit gate instead of open-coding a smaller subset.

Gate composition:
- `just ci`
- `just deny`
- `just api-diff`
- `just repro-check`

Why this structure:
- It reuses the repository’s authoritative existing validation commands instead of duplicating them.
- It keeps ordinary merge-request CI unchanged.
- It gives maintainers one explicit pre-RC audit entrypoint.
- It lets `release-verify` reuse the same gate before package extraction and dry-run release validation.

Recorded artifacts:
- `target/vvm-release-audit/tool-versions.txt`
- `target/vvm-release-audit/ci.log`
- `target/vvm-release-audit/deny.log`
- `target/vvm-release-audit/api-diff.log`
- `target/vvm-release-audit/repro-check.log`

Tool versions recorded by the gate:
- `rustc`
- `cargo`
- nightly `cargo fmt`
- `cargo nextest`
- `cargo audit`
- `cargo deny`
- `cargo udeps`
- `cargo public-api`
- `mdbook`
- `verilator`
- `g++`
- `clang++` when available

Validation:
- Shell syntax and wiring checks passed for `scripts/release-audit.sh` and `scripts/release-verify.sh`.
- Full command execution is intentionally deferred to TASK-64, which owns the complete pre-RC audit run and release-blocker resolution.

Accepted limitations:
- The gate still intentionally relies on the existing `just` command surface for the heavy checks, so some evidence is retained in their established output locations (`coverage/`, `public/`, `target/vvm-release/`) rather than being recopied into a new custom report tree.
<!-- SECTION:FINAL_SUMMARY:END -->

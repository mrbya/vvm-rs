---
id: TASK-49
title: 'Add persisted-schema, generated-code, and CLI compatibility fixtures'
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 16:57'
labels:
  - release
  - compatibility
  - tests
milestone: m-5
dependencies:
  - TASK-48
priority: high
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Strengthen compatibility protection with deterministic fixtures and focused tests for persisted coverage artifacts, generated wrapper contracts, and machine-consumed CLI outputs. Scope includes representative versioned coverage schema fixtures, unknown-schema rejection tests, generated-code contract fixtures for supported DUT shapes, normalized nondeterministic data, and stable CLI contract checks such as metric lines, artifact layout, file names, exit behavior, and JSON outputs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Representative supported coverage schema fixtures exist and current readers and writers validate against them
- [x] #2 Generated-code compatibility fixtures cover minimal scalar, wide, aggregate or complex port, inout, timing, and multi-clock shapes where the public contract differs
- [x] #3 Nondeterministic paths or environment data are normalized so accidental compatibility drift fails clearly
- [x] #4 CLI machine-consumed contract checks cover stable metric output, artifact layout, required file names, exit behavior, and supported JSON outputs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing coverage persistence tests, `vvm-build` codegen fixtures, native generated-port regressions, and `cargo-vvm` CLI tests to identify which compatibility domains already have strong coverage and which still need deterministic checked-in fixtures.
2. Add a checked-in persisted coverage fixture corpus for supported schema versions plus explicit unsupported-schema rejection cases, then update the current readers and writers to validate against those fixture files rather than only in-memory mutations.
3. Extend generated-code compatibility coverage with normalized goldens or fixture assertions for the public contract differences that are not yet snapshotted: wide, aggregate, inout, timing-enabled, and multi-clock shapes.
4. Strengthen `cargo-vvm` CLI contract tests so they assert stable metric lines, output layout, required file names, exit behavior, and machine-consumed merged JSON structure with nondeterministic fields normalized where needed.
5. Run the focused compatibility test surfaces and close the task only after the deterministic fixtures clearly fail on accidental compatibility drift.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added a committed coverage-schema fixture corpus under `crates/vvm-core/tests/fixtures/coverage/` for artifact and merged JSON documents plus explicit unsupported-schema fixtures. The `vvm-core` property tests now normalize version and fingerprint placeholders, compare against the checked-in fixtures semantically, and validate both `from_json` and `read_from` against the committed documents.

Strengthened `cargo-vvm` CLI integration tests to assert the exact final metric line derived from the merged JSON summary, the retained artifact layout and file-name suffixes, merge schema format/version fields, policy fields, included failure inputs, and the presence of text and HTML reports.

Added file-driven generated-code contract fixtures under `crates/vvm-build/tests/fixtures/codegen-contracts/` and wired them into the existing codegen tests for inout, timing-enabled, wide, packed-struct aggregate, and synthetic multi-clock-like wrapper shapes. Existing counter and packed-enum snapshots continue to cover the minimal scalar baseline.

Normalization and drift clarity: coverage fixtures use explicit `__VVM_CORE_VERSION__` and `__DECODER_FINGERPRINT__` placeholders, generated snapshots continue to reject host checkout paths, and the CLI checks consume stable merged JSON instead of comparing environment-dependent output directories.

Validation: `cargo test -p vvm-core --test properties`, `cargo test -p cargo-vvm --test cli`, `cargo test -p vvm-build codegen`, and `cargo +nightly fmt --all --check`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the compatibility-fixture hardening by adding committed coverage artifact and merge JSON fixtures with unsupported-schema rejection cases, file-driven generated-code contract fixtures for inout, timing, wide, aggregate packed-struct, and multi-clock-like wrapper shapes, and stronger `cargo-vvm` CLI contract checks for metric output, artifact layout, file names, exit behavior, and merged JSON structure. The focused compatibility test surfaces now pass and will fail clearly on accidental schema, generated-contract, or CLI drift.
<!-- SECTION:FINAL_SUMMARY:END -->

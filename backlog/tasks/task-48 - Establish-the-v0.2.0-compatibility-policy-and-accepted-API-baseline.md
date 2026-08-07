---
id: TASK-48
title: Establish the v0.2.0 compatibility policy and accepted API baseline
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 16:37'
labels:
  - release
  - compatibility
  - api
milestone: m-5
dependencies:
  - TASK-47
priority: high
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define the canonical public compatibility policy for the v0.2.0 cycle and accept the reviewed post-12.6 API surface as the baseline for future checks. Scope includes Rust API policy, patch compatibility within 0.2.x, persisted coverage schema policy, generated-code compatibility policy, CLI machine-consumed compatibility policy, public API inventory review, accidental export review, and contributor-facing API-diff tooling and commands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A canonical compatibility policy documents Rust API, pre-1.0 semver, 0.2.x patch compatibility, coverage schema, generated-code, and CLI compatibility rules
- [x] #2 The reviewed current public API is accepted as the v0.2.0 baseline and any accidental exports are either removed with justification or explicitly accepted
- [x] #3 Contributor commands and tooling can compare a branch or later release candidate against an explicit v0.2.0 baseline without depending on a nonexistent v0.2.0 tag
- [x] #4 User-facing book content and package docs do not contradict the compatibility policy
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current compatibility and public-API material across `docs/dev/public-api.md`, the checked-in API baseline under `docs/dev/api/0.2.0`, the public book pages, and contributor command surfaces in `justfile`.
2. Consolidate the compatibility promise into one canonical maintainer-facing policy that covers Rust API stability, pre-1.0 semver handling, `0.2.x` patch compatibility, persisted coverage schema rules, generated-code contract rules, and machine-consumed CLI expectations.
3. Wire contributor API-diff commands to the checked-in `v0.2.0` baseline instead of requiring a published tag, and remove or clarify any stale docs that contradict the accepted compatibility policy.
4. Re-run focused doc and command validation, then close the task with the accepted API baseline and policy references recorded in Backlog.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Expanded `docs/dev/public-api.md` into the canonical `v0.2.0` compatibility policy, covering the release-cycle promise, `0.2.x` patch compatibility, persisted coverage schema rules, generated-wrapper contract boundaries, CLI machine-consumed guarantees, the unsupported `vvm::__private` boundary, and the accepted baseline file at `docs/dev/api/0.2.0`.

Updated `docs/coverage-json-v1.md` so the persisted-schema guidance matches the implemented deterministic merge behavior and makes the actual compatibility rule explicit: schema support plus compatible reviewed fingerprints.

Replaced the tag-oriented `just api-diff` flow with a baseline-file diff against `docs/dev/api/0.2.0`, and extended `just init` to install `cargo-public-api` so contributors can run the inventory and baseline diff workflow directly.

Validation: `just --list` and `mdbook build docs/book`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Established `docs/dev/public-api.md` as the canonical `v0.2.0` compatibility policy, explicitly accepted `docs/dev/api/0.2.0` as the release-line API baseline, and switched contributor API diffing from an assumed release tag to the checked-in baseline file. The policy now covers Rust API stability, pre-1.0 semver handling, `0.2.x` patch compatibility, persisted coverage schemas, generated wrapper contracts, and machine-consumed `cargo-vvm` behavior, with stale schema wording corrected so public docs no longer contradict the accepted policy.
<!-- SECTION:FINAL_SUMMARY:END -->

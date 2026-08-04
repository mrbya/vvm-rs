---
id: TASK-5
title: Migrate functional coverage documentation
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-29 14:12'
updated_date: '2026-08-04 14:51'
labels: []
milestone: m-2
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Move functional-coverage concepts and persisted artifact guidance into the book while retaining compatibility pointers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Functional Coverage appears as one substantial Guide chapter
- [x] #2 The old multi-page Functional Coverage section is removed from normal navigation
- [x] #3 The chapter includes a complete generalized example before low-level schema details
- [x] #4 Bins coverpoints typed models crosses artifacts merge behavior reports and CI are all covered
- [x] #5 Legacy coverage URLs remain usable where practical and all snippets are validated
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current `coverage/` pages, example sources, and coverage APIs to collect the material that belongs in one user-facing Guide chapter.
2. Write a single `guide/functional-coverage.md` chapter that starts with a generalized end-to-end model, then teaches bins, coverpoints, typed models, derives, sampling, crosses, testbench attachment, artifacts, merge behavior, reporting, CI, and common mistakes.
3. Keep low-level schema material available outside normal navigation and preserve legacy coverage URLs through compatibility redirects or pointer pages.
4. Update inbound links so the Guide becomes the authoritative user-facing coverage entry point, with API Guide and schema links for exact contracts.
5. Validate the new chapter with docs builds, link checks, and the relevant coverage documentation examples.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current coverage chapter is only a summary and does not yet satisfy the intended depth for progressive coverage authoring, sessions, artifacts, merging, reporting, CI, and troubleshooting.

Reopened for the final 12.5 refinement pass to consolidate the fragmented Functional Coverage section into one substantial Guide chapter while keeping schema material accessible through compatibility pointers and low-level links.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Consolidated user-facing functional coverage documentation into one substantial Guide chapter at `docs/book/src/guide/functional-coverage.md`. The new chapter starts with a generalized end-to-end typed coverage model, then covers bins, coverpoints, typed models, `ObservedCycle`, crosses, testbench attachment, per-test artifacts, fingerprints, provenance, merge compatibility, reports, failure behavior, CI integration, and common mistakes. Added fixture-backed coverage documentation anchors in `tests/fixtures/docs-quick-start/src/lib.rs` for the typed model, a covered test, and coverage-snapshot inspection. Kept the low-level schema available at `docs/book/src/coverage/schema.md` and preserved old coverage URLs through redirects in `scripts/docs-site.sh`. Validation: `just docs-book`, `just docs-links`, `just docs-test`, the dedicated docs quick-start fixture, `just test-fast`, `just test-cov-ci`, and `just ci` passed.
<!-- SECTION:FINAL_SUMMARY:END -->

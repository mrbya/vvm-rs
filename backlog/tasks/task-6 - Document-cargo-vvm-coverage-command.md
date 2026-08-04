---
id: TASK-6
title: Document cargo-vvm coverage command
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
Create book-only cargo-vvm command documentation covering syntax outputs policies and failure semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 cargo-vvm appears as one substantial Guide chapter
- [x] #2 Normal cargo-vvm usage can be understood from that one chapter
- [x] #3 The chapter documents command restrictions outputs failures merge behavior and CI workflow
- [x] #4 Old fragmented cargo-vvm pages are removed from normal navigation while legacy URLs remain usable where practical
- [x] #5 Commands and examples are validated against the current CLI behavior
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing `cargo-vvm/` pages, CLI implementation, and package README to capture the real command surface and workflow details.
2. Write one `guide/using-cargo-vvm.md` chapter that explains what the command does, when to use it, installation, the normal `cargo vvm coverage -- ...` workflow, output layout, merge policies, reporting, failure semantics, restrictions, and troubleshooting.
3. Keep the package README and CLI terminology aligned with the new guide chapter.
4. Preserve old `cargo-vvm` URLs through compatibility redirects or pointer pages instead of keeping a fragmented visible section.
5. Validate the chapter against the actual CLI and the documentation/test workflows.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current cargo-vvm documentation is not yet a full workflow and command reference at the depth implied by the task acceptance criteria.

Reopened for the final 12.5 refinement pass to consolidate fragmented cargo-vvm documentation into one substantial Guide chapter with practical workflow, outputs, failure semantics, and CI guidance.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Consolidated fragmented cargo-vvm documentation into one substantial Guide chapter at `docs/book/src/guide/using-cargo-vvm.md`. The chapter now covers purpose, when to use it, installation, the normal `cargo vvm coverage -- ...` workflow, child-command restrictions, nextest retry restrictions, output-directory requirements, output tree, merge policies, fingerprints, provenance controls, bin-detail controls, failure semantics, no-artifact behavior, GitLab CI usage, troubleshooting, and the command-summary surface. Updated `crates/cargo-vvm/README.md` so its documentation links now point at the new Guide chapter and the Functional Coverage guide. Preserved old cargo-vvm URLs with compatibility redirects in `scripts/docs-site.sh`. Validation: checked against `cargo run -p cargo-vvm -- coverage --help`, plus `just docs-book`, `just docs-links`, `just test-fast`, `just test-cov-ci`, and `just ci`, all passing.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-50
title: Complete CHANGELOG.md and add a maintainer-grade CONTRIBUTING.md
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:48'
updated_date: '2026-08-06 16:40'
labels:
  - release
  - docs
milestone: m-5
dependencies:
  - TASK-51
  - TASK-47
  - TASK-48
priority: high
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Finish the repository-level release and contributor documentation required for the v0.2.0 line. Scope includes a substantive Unreleased changelog covering user-visible work since v0.1.0-alpha.1 and a complete CONTRIBUTING.md with contributor setup, workflow expectations, package validation, changelog expectations, API review expectations, defect reporting guidance, and a clearly separated maintainer release workflow with publication order, dry-run verification, recovery guidance, yanking policy, and protected-variable requirements.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 CHANGELOG.md keeps the historical 0.1.0-alpha.1 entry accurate and Unreleased reflects significant user-visible work since that alpha release
- [x] #2 CONTRIBUTING.md exists and documents contributor setup, Backlog.md workflow, validation expectations, package validation, changelog expectations, and public API review expectations
- [x] #3 CONTRIBUTING.md includes a clearly separated maintainer release workflow covering development-version progression, publication order, dry-run validation, protected credentials, partial-publication recovery, and yanking policy
- [x] #4 No RELEASING.md or SECURITY.md is introduced and no unsupported support promises are made
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current repository-level `CHANGELOG.md`, `CONTRIBUTING.md`, and the related book contributor chapter so the release and contributor guidance can be consolidated without introducing extra release/security documents.
2. Expand `CHANGELOG.md` with a substantive Unreleased section that truthfully summarizes the user-visible work since `v0.1.0-alpha.1` while keeping the historical alpha entry intact.
3. Rewrite `CONTRIBUTING.md` into a maintainer-grade guide with contributor setup, Backlog workflow, validation expectations, package validation, changelog and public-API review expectations, and a clearly separated maintainer release workflow for the `v0.2.0` line.
4. Rebuild the book documentation or run focused markdown validation as needed, then close the task with the final documentation boundaries recorded in Backlog.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Rewrote the repository-level `CONTRIBUTING.md` into a maintainer-grade guide covering setup, Backlog workflow, validation expectations, package validation, changelog expectations, public API review against `docs/dev/api/0.2.0`, defect reporting, and a clearly separated maintainer release workflow for the `v0.2.0` line.

Expanded `CHANGELOG.md` with a substantive `Unreleased` section summarizing the user-visible additions, compatibility-policy changes, package-validation work, and coverage/reporting improvements since `v0.1.0-alpha.1`, while preserving the historical alpha entry unchanged.

Validation: `mdbook build docs/book` plus grep checks confirming no `RELEASING.md` or `SECURITY.md` document was introduced.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the repository-level release documentation for the `v0.2.0` line by adding a real `Unreleased` changelog section and rewriting `CONTRIBUTING.md` into a contributor-and-maintainer guide. The new guidance now covers Backlog-driven work, validation depth, package and API review expectations, defect reporting, publication order, dry-run verification, protected credential policy, partial-publication recovery, and yanking policy without introducing separate release or security documents.
<!-- SECTION:FINAL_SUMMARY:END -->

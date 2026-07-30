---
id: TASK-19
title: Redesign the mdBook information architecture for milestone 12.5
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 10:22'
labels: []
milestone: m-0
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current flat mdBook structure with a hierarchical navigation model that supports orientation, workflows, API guidance, examples, reference material, and development material. Preserve practical existing URLs where possible, add pointer pages or redirects where necessary, and keep every chapter substantive rather than introducing placeholders.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 SUMMARY.md uses a hierarchical structure with substantial chapters
- [x] #2 Navigation supports progression from beginner orientation to maintainer architecture
- [x] #3 Existing published paths are preserved or redirected where practical
- [x] #4 Cross-links are updated to the new structure and remain valid
- [x] #5 The rebuilt book passes mdBook build and documentation link validation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Create the new section directories under `docs/book/src/` and replace the flat `SUMMARY.md` with hierarchical navigation covering orientation, concepts, workflows, API guidance, coverage, cargo-vvm, examples, reference, and development material.
2. Replace the current single-page overview chapters with either substantive content chapters or short compatibility-pointer pages only where preserving published links remains practical.
3. Update cross-links so the book, README surfaces, and package READMEs point to the new canonical paths.
4. Keep every chapter referenced by `SUMMARY.md` substantial by coordinating content migration with the related user, API, reference, example, coverage, and development tasks.
5. Validate the final structure with `just docs-book` and `just docs-links` after the content pass is complete.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The information-architecture task is the active umbrella while the book is restructured and all new chapter paths are created. Related content tasks now have recorded plans and will be finalized against the resulting hierarchy.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced the flat mdBook with a hierarchical `SUMMARY.md` and new `concepts/`, `guide/`, `api-guide/`, `coverage/`, `cargo-vvm/`, `examples/`, `reference/`, and `development/` sections. Preserved practical legacy published entry points by keeping `getting-started.md`, `user-guide.md`, `coverage.md`, `cargo-vvm.md`, `examples.md`, `reference.md`, `developer-guide.md`, and `contributing.md` as substantive overview pages rather than breaking those URLs. Updated cross-links to the new structure and validated the result with `just docs-book` and `just docs-links`.
<!-- SECTION:FINAL_SUMMARY:END -->

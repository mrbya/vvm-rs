---
id: TASK-19
title: Redesign the mdBook information architecture for milestone 12.5
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 17:33'
labels: []
milestone: Milestone 12.5 — User-centred documentation rewrite
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
1. Rewrite the front of the book around an HDL-user learning journey: Introduction, Why VVM, How VVM Fits Into HDL Verification, How VVM Works, Rust Essentials For HDL Engineers, Installation, and Quick Start.
2. Update `SUMMARY.md` so visible navigation reflects that learning journey before concepts, guides, API guide, examples, reference, and development material.
3. Repurpose compatibility pages such as `getting-started.md` and `user-guide.md` so they remain useful landing pages or hidden compatibility pointers without exposing milestone or migration history.
4. Update section landing pages and cross-links so examples are explicitly framed as case studies rather than the primary teaching path.
5. Validate the structural rewrite with mdBook build and link checks after the related content chapters are in place.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The information-architecture task is the active umbrella while the book is restructured and all new chapter paths are created. Related content tasks now have recorded plans and will be finalized against the resulting hierarchy.

Reopened because the navigation still exposes process-oriented overview pages and does not yet provide the required early-book learning journey of Introduction, Why VVM, HDL workflow fit, runtime model, Rust essentials, Installation, and Quick Start.

Added the missing early-book learning journey chapters (`why-vvm.md`, `how-vvm-fits-into-hdl-verification.md`, `how-vvm-works.md`, and `rust-essentials-for-hdl-engineers.md`), updated `SUMMARY.md`, and repurposed the long-lived landing pages so they no longer expose milestone or migration history.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the final information-architecture pass for milestone 12.5. The visible front of the book now follows an HDL-user learning path through Introduction, Why VVM, HDL workflow fit, runtime model, Rust essentials, Installation, and Quick Start before the deeper concepts, guide, API, example, reference, and development sections. Updated `SUMMARY.md`, repurposed compatibility landing pages such as `getting-started.md` and `user-guide.md` into useful navigational pages, and removed user-facing process-history language from the navigation path. Validated the resulting structure with mdBook build, docs-site assembly, and docs-links checks.
<!-- SECTION:FINAL_SUMMARY:END -->

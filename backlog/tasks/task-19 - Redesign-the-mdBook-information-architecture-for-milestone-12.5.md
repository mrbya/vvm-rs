---
id: TASK-19
title: Redesign the mdBook information architecture for milestone 12.5
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-08-04 14:51'
labels: []
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current flat mdBook structure with a hierarchical navigation model that supports orientation, workflows, API guidance, examples, reference material, and development material. Preserve practical existing URLs where possible, add pointer pages or redirects where necessary, and keep every chapter substantive rather than introducing placeholders.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Public navigation reflects the final introduction concepts guide API guide examples and development model without visible standalone Functional Coverage cargo-vvm or Reference sections
- [x] #2 No visible empty overview page remains in public navigation
- [x] #3 Every moved page has updated inbound links from the book README surfaces and compatibility landing pages
- [x] #4 Old public URLs needed by the site and published docs are preserved through redirects or pointer pages where practical
- [x] #5 The rebuilt book and link validation pass against the assembled site
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the visible top-level coverage, cargo-vvm, and reference sections in `SUMMARY.md` with their new Guide destinations, while keeping Examples and Development intact.
2. Fix stale markdown links that still point to removed root pages or the old visible reference structure.
3. Preserve old published entry points such as `getting-started.html`, `user-guide.html`, `coverage.html`, `cargo-vvm.html`, and `reference.html` by generating lightweight compatibility redirect or pointer pages during site assembly instead of keeping them visible in navigation.
4. Add compatibility mappings for removed reference, coverage, and cargo-vvm URLs as the related content moves into Guide chapters.
5. Rebuild the assembled site and rerun docs link validation once the structure and compatibility outputs are in place.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The information-architecture task is the active umbrella while the book is restructured and all new chapter paths are created. Related content tasks now have recorded plans and will be finalized against the resulting hierarchy.

Reopened because the navigation still exposes process-oriented overview pages and does not yet provide the required early-book learning journey of Introduction, Why VVM, HDL workflow fit, runtime model, Rust essentials, Installation, and Quick Start.

Added the missing early-book learning journey chapters (`why-vvm.md`, `how-vvm-fits-into-hdl-verification.md`, `how-vvm-works.md`, and `rust-essentials-for-hdl-engineers.md`), updated `SUMMARY.md`, and repurposed the long-lived landing pages so they no longer expose milestone or migration history.

Reopened for the final 12.5 refinement pass to remove the visible standalone Reference, Functional Coverage, and cargo-vvm top-level sections from public navigation and to finish compatibility-pointer maintenance.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the final information-architecture pass for milestone 12.5. Updated `docs/book/src/SUMMARY.md` so public navigation now centers on Introduction, Concepts, Guide, API Guide, Examples, and Development only. Removed visible standalone Functional Coverage, cargo-vvm, and Reference sections from navigation; moved their canonical destinations into the Guide; and removed visible Concepts/API/Guide overview placeholders from navigation. Added assembled-site compatibility redirects in `scripts/docs-site.sh` for `getting-started.html`, `user-guide.html`, old `coverage/`, old `cargo-vvm/`, and old `reference/` public URLs. Fixed stale source links in `docs/book/src/guide/user-guide.md`, `docs/book/src/api-guide/overview.md`, and other moved chapters. Validation: `just docs-book`, `just docs-links`, explicit `public/` entry-point checks, and the full repository gate including `just ci` passed.
<!-- SECTION:FINAL_SUMMARY:END -->

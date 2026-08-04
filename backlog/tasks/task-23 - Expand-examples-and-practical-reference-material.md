---
id: TASK-23
title: Expand examples and practical reference material
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
Turn the curated examples into a real learning ladder inside the book and add practical reference chapters for configuration, environment variables, generated type mappings, execution order, diagnostics, terminology, compatibility, artifacts, and limitations. Keep one authoritative limitations chapter and document the observable runtime order and generated HDL-to-Rust contracts clearly.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The standalone Reference section is removed from normal navigation
- [x] #2 Configuration environment variables generated type mapping execution order diagnostics and compatibility information all have clear Guide homes
- [x] #3 Generated Type Mapping Execution Order Diagnostics and Troubleshooting and Compatibility and Limitations are substantial practical Guide chapters
- [x] #4 No important reference information is lost and duplicated authoritative prose is removed
- [x] #5 Old Reference URLs remain usable where practical through redirects or pointer pages
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current `reference/` pages and map each user-facing topic to its new Guide destination, keeping environment-variable detail either inside `configuring-tests.md` or in one adjacent Guide page if needed.
2. Create substantial Guide chapters for generated type mapping, execution order, and centralized compatibility/limitations, and upgrade troubleshooting into a practical diagnostics-and-troubleshooting chapter.
3. Merge or absorb configuration, environment-variable, artifact-layout, terminology, and compatibility material into the relevant Guide chapters so the Guide becomes the single user manual.
4. Remove the visible standalone Reference section from `SUMMARY.md` and preserve the old reference URLs through compatibility redirects or pointer pages during site assembly.
5. Re-audit inbound links and validate that no important reference material was dropped.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened because the reference surfaces and example positioning need another pass to centralize limitations, complete tables, and ensure curated examples remain supporting case studies rather than the primary explanation path.

Expanded the practical reference pages for configuration, environment variables, generated types, execution order, diagnostics, terminology, compatibility, and the centralized limitations/support policy. Also reframed `examples.md`, `coverage.md`, and `cargo-vvm.md` so those sections act as supporting learning surfaces rather than replacing the generalized guide.

Reopened for the final 12.5 pass to dissolve the standalone Reference section into the Guide, centralize compatibility and limitation guidance, and preserve old public URLs with compatibility pointers where practical.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Dissolved the standalone Reference section into the Guide. Added substantial Guide chapters at `docs/book/src/guide/generated-type-mapping.md`, `execution-order.md`, and `compatibility-and-limitations.md`; expanded `guide/configuring-tests.md` with configuration-layer and environment-variable tables; and upgraded `guide/troubleshooting.md` into the practical Diagnostics And Troubleshooting page. Rewrote hidden compatibility landing pages `docs/book/src/reference.md`, `coverage.md`, and `cargo-vvm.md` so they now point to the new authoritative Guide chapters instead of duplicating old section prose. Updated inbound links from existing Guide chapters to the new Guide destinations and preserved old Reference URLs through redirects in `scripts/docs-site.sh`. Validation: `just docs-book`, `just docs-links`, `just docs-test`, the docs fixtures, `just test-fast`, `just test-cov-ci`, and `just ci` passed. Retained limitation: the low-level coverage schema remains visible under Development because mdBook does not support a truly hidden rendered page.
<!-- SECTION:FINAL_SUMMARY:END -->

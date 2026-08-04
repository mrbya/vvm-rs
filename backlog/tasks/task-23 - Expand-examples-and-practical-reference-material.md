---
id: TASK-23
title: Expand examples and practical reference material
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 17:33'
labels: []
milestone: m-1
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Turn the curated examples into a real learning ladder inside the book and add practical reference chapters for configuration, environment variables, generated type mappings, execution order, diagnostics, terminology, compatibility, artifacts, and limitations. Keep one authoritative limitations chapter and document the observable runtime order and generated HDL-to-Rust contracts clearly.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every curated example has a conceptual chapter with architecture run commands important files and next steps
- [x] #2 Configuration defaults precedence and path behavior are documented in reference form
- [x] #3 Environment variables have a complete table with format defaults precedence effects and errors
- [x] #4 Generated type mappings and unsupported HDL shapes are documented
- [x] #5 Execution order diagnostics terminology compatibility artifacts and limitations are documented practically
- [x] #6 Limitations are centralized and remain consistent across the book and README surfaces
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Strengthen the reference and example-positioning surfaces so examples remain case studies and the practical lookup pages are authoritative.
2. Expand configuration, environment, generated-type, execution-order, diagnostics, terminology, and limitations references to the level needed for day-to-day lookup.
3. Keep one central support-and-limitations page and make shorter compatibility pages defer to it.
4. Reframe example overview pages so they follow the generalized guide instead of replacing it.
5. Validate the updated reference surfaces with mdBook build, docs link checks, and the relevant documentation fixtures before finalizing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened because the reference surfaces and example positioning need another pass to centralize limitations, complete tables, and ensure curated examples remain supporting case studies rather than the primary explanation path.

Expanded the practical reference pages for configuration, environment variables, generated types, execution order, diagnostics, terminology, compatibility, and the centralized limitations/support policy. Also reframed `examples.md`, `coverage.md`, and `cargo-vvm.md` so those sections act as supporting learning surfaces rather than replacing the generalized guide.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the practical reference and example-positioning pass. Rewrote the key `reference/` pages so configuration layers, environment variables, generated type mappings, execution order, diagnostics, terminology, compatibility, and the authoritative limitations/support policy are now usable as lookup material rather than sparse summaries. Reframed the examples, coverage, and cargo-vvm overview pages so they point readers back to the generalized workflow first and present the curated examples as supporting case studies. Validated the updated reference surfaces with mdBook build, docs-site assembly, docs-links checks, and the dedicated documentation fixtures.
<!-- SECTION:FINAL_SUMMARY:END -->

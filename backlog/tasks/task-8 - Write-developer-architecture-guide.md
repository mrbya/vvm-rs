---
id: TASK-8
title: Write developer architecture guide
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-29 14:12'
updated_date: '2026-07-30 10:22'
labels: []
milestone: m-0
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Document VVM workspace boundaries, build/runtime pipeline, generated ABI, scheduling, coverage, macros, diagnostics, and unsafe invariants.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Workspace and generation pipeline are documented
- [x] #2 Runtime lifecycle schedulers coverage and macros are documented
- [x] #3 FFI ownership safety and diagnostic policies are explicit
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the compressed developer summary with architecture chapters covering workspace layout, crate map, build pipeline, code generation, runtime lifecycle, scheduler architecture, coverage architecture, macro architecture, FFI/safety invariants, errors/diagnostics, testing, fixtures, examples, documentation, benchmarking, compatibility, and releases.
2. Base the crate map on the current workspace crates and their actual responsibilities, dependencies, and test locations.
3. Explain the end-to-end build pipeline from consumer `build.rs` through Verilator, metadata normalization, code generation, native compilation, and `OUT_DIR` inclusion.
4. Explain the runtime, scheduler, coverage, macro, and FFI ownership boundaries in enough detail for contributors to place changes correctly.
5. Link out to the authoritative `docs/dev/*` strategy documents where they remain the maintainer source of truth.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current developer guide is too compressed for the stated architecture, pipeline, scheduler, coverage, macro, and FFI documentation scope.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded the development material into dedicated chapters for setup, common commands, architecture, crate map, build pipeline, code generation, runtime, scheduler architecture, coverage architecture, macro architecture, FFI/safety, errors and diagnostics, testing, fixtures, examples, documentation maintenance, benchmarking, compatibility, and releases. The contributor-facing architecture map now explains where changes belong before maintainers edit code.
<!-- SECTION:FINAL_SUMMARY:END -->

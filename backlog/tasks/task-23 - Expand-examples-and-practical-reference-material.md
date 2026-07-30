---
id: TASK-23
title: Expand examples and practical reference material
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
1. Build a reference section covering configuration, environment variables, generated type mappings, execution order, artifact layout, compatibility, diagnostics, terminology, and limitations.
2. Use table-heavy reference pages where appropriate, especially for environment variables and configuration defaults/precedence.
3. Document supported HDL-to-Rust mappings and unsupported shapes using the generated-port fixtures and maintained examples as backing evidence.
4. Centralize current limitations in one chapter and align the rest of the book and README surfaces to that chapter.
5. Add practical diagnostic examples for build failures, simulation mismatches, timing limits, coverage incompatibility, and reporting failures.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded the book's examples and reference material by adding dedicated chapters for every curated example plus a `reference/` section covering configuration, environment variables, generated types, execution order, artifact layout, compatibility, diagnostics, terminology, and limitations. Centralized the authoritative limitations list in `reference/limitations.md` and added the environment-variable and execution-order material that previously lived only in the root README or scattered examples.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-13
title: Write contributor and maintainer guide
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-29 14:12'
updated_date: '2026-07-30 17:33'
labels: []
milestone: m-1
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Document contributor setup commands policies testing fixtures examples documentation compatibility benchmarks and releases using VVM-specific examples.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Contributor workflows and policies are documented
- [x] #2 Testing fixture example and documentation policies are linked
- [x] #3 Rustdoc style uses VVM concepts
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Expand contributor documentation into setup, command reference, testing categories, native requirements, fixtures, examples, coverage workflows, documentation workflows, CI-equivalent validation, style policies, Backlog workflow, and PR expectations.
2. Derive all command tables from the current `justfile` and `docs/dev/testing-strategy.md` so the book stays aligned with the real repository commands.
3. Add source-of-truth guidance covering where to update README content, book tutorials, rustdoc, package READMEs, example READMEs, compatibility claims, and persisted schema documentation.
4. Explain contribution obligations for new public APIs, macros, generated type support, examples, fixtures, and compatibility changes.
5. Link to the existing maintainer strategy documents and keep the chapter practical rather than aspirational.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current contributing chapter is too brief to satisfy the intended contributor and maintainer workflow scope.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded contributor documentation into a practical command and policy guide covering setup, common commands, testing categories, docs workflows, source-of-truth policy, compatibility expectations, and contributor responsibilities. The preserved `contributing.md` path now works as a real entry point into the deeper development documentation rather than as a shallow summary page.
<!-- SECTION:FINAL_SUMMARY:END -->

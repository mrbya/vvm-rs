---
id: TASK-9
title: Harden public and internal rustdoc builds
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-29 14:23'
labels: []
milestone: m-0
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Improve public crate docs and provide separate strict public and maintainer rustdoc builds with bidirectional book navigation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Public rustdoc excludes private items and binary crates
- [ ] #2 Internal private-item rustdoc is separate
- [ ] #3 Public crate roots link to the book and build cleanly
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Improve public crate roots and provide strict public and internal rustdoc commands with book navigation.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded all public crate roots with roles and book links. Added strict `docs-api` and separately private-item `docs-internal` commands; public site excludes examples and cargo-vvm.
<!-- SECTION:FINAL_SUMMARY:END -->

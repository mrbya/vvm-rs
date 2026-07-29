---
id: TASK-3
title: Write getting-started guide
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-29 14:21'
labels: []
milestone: m-0
dependencies: []
documentation:
  - examples/counter/README.md
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Document a runnable first VVM project using maintained counter example APIs and accurate prerequisites.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Purpose requirements installation and platform support are documented
- [x] #2 First project contains complete coherent source files
- [x] #3 Guide validates through the book build
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Write a source-aligned first-project chapter with supported toolchain constraints and validate the book.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added getting-started guidance covering purpose, support matrix, installation, build.rs, generated inclusion, and the maintained counter workflow. Validated with `just docs-book` and `just docs-test`.
<!-- SECTION:FINAL_SUMMARY:END -->

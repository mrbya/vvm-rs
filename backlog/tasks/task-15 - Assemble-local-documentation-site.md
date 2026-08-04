---
id: TASK-15
title: Assemble local documentation site
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-30 17:33'
labels: []
milestone: m-1
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Assemble book public rustdoc and authoritative build information into a checked public tree with matching local commands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 public contains book API and build info
- [x] #2 Required output paths are asserted
- [x] #3 docs commands use shared assembly implementation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Use one script to construct public book, API, build metadata, and output assertions.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added `scripts/docs-site.sh` and docs-site/docs-clean/docs-serve commands. It creates public book content, public API reference, API landing page, build-info.json, and validates required paths.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-55
title: Build the complete release-audit gate
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:57'
updated_date: '2026-08-11 14:23'
labels: []
milestone: m-6
dependencies: []
documentation:
  - justfile
  - scripts/release-verify.sh
  - .gitlab-ci.yml
  - CONTRIBUTING.md
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create an explicit maintainer-facing release audit command that composes existing validation and release verification rather than duplicating it. The gate must cover release-critical formatting, linting, tests, docs, dependency checks, API checks, reproducibility checks, package verification, release dry-run validation, and tool-version capture while keeping ordinary CI proportionate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A dedicated release-audit command exists
- [x] #2 The command composes existing validation instead of duplicating it
- [x] #3 It runs every release-critical audit or delegates to the current authoritative script
- [x] #4 Tool versions are recorded
- [x] #5 Failures are actionable
- [x] #6 Ordinary CI is not unnecessarily bloated
- [x] #7 Release verification can invoke the audit without publishing anything
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Identify whether this task represents unique milestone 12.8 scope or an MCP duplicate created during the initial backlog setup.
2. If duplicate, map it to the authoritative task records that executed the real work.
3. Close the duplicate record explicitly so milestone 12.8 contains one authoritative task per workstream.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
This task is a duplicate placeholder created during the initial MCP bulk-create glitch. The actual reproducibility work lives in TASK-59 and the actual end-to-end release-audit execution lives in TASK-64.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Administrative closure only. This task was created accidentally during the initial milestone 12.8 backlog setup glitch and duplicates scope that was executed under the authoritative tasks.

Authoritative tasks:
- reproducibility audit and automation: `TASK-59`
- complete pre-RC release audit and dry-run verification: `TASK-64`

No independent implementation, validation, or release decision was performed under `TASK-55`. It is closed so the milestone audit has one authoritative task record per workstream.
<!-- SECTION:FINAL_SUMMARY:END -->

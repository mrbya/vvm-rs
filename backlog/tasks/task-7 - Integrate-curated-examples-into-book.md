---
id: TASK-7
title: Integrate curated examples into book
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
Document the maintained learning ladder using links and tested source excerpts instead of duplicating example READMEs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All five curated examples have substantive chapters
- [x] #2 Learning progression and next steps are explicit
- [x] #3 Chapters link to maintained example sources and READMEs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the one-page example summary with a dedicated chapter for each curated example: counter, synchronous FIFO, timed UART, asynchronous FIFO, and tri-state bus.
2. For each chapter, explain the DUT, verification problem, intended reader level, VVM concepts introduced, architecture, model, scoreboard, timing or clock behavior, coverage behavior, trace/replay behavior, limitations, files to study, run commands, and next steps.
3. Use source-backed snippets or links to maintained source files rather than copying whole README files into the book.
4. Keep the local example READMEs as execution guides while using the book chapters to explain concepts and progression.
5. Validate chapter links and commands against the example crates and README files.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The example chapter blurbs exist, but the curated examples are not yet integrated as full conceptual learning-ladder chapters with architecture, files to study, limitations, and next steps.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Turned the examples chapter into a real learning ladder with dedicated chapters for counter, synchronous FIFO, timed UART, asynchronous FIFO, and tri-state bus. Each chapter now states the DUT purpose, introduced concepts, key files to study, run commands, and where to go next, while leaving the local example READMEs as the detailed execution guides. The book now treats the examples as progressive documentation rather than as a one-page list.
<!-- SECTION:FINAL_SUMMARY:END -->

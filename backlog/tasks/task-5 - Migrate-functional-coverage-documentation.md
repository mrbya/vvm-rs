---
id: TASK-5
title: Migrate functional coverage documentation
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
Move functional-coverage concepts and persisted artifact guidance into the book while retaining compatibility pointers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Coverage concepts sessions artifacts merging and reporting are documented
- [x] #2 Schema-v1 contract remains directly accessible
- [x] #3 Legacy coverage paths remain useful compatibility pointers
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the current single coverage overview with a progressive coverage section covering bins, coverpoints, typed models, crosses, sampling, sessions, artifacts, schema, merging, reporting, CI usage, and troubleshooting.
2. Use the counter and synchronous-FIFO examples to show meaningful coverage model design rather than only API construction.
3. Keep the schema-v1 contract directly accessible and explain fingerprints, provenance, merge compatibility, and partial results.
4. Coordinate coverage authoring chapters with the `cargo-vvm` guide and the API guide so authoring, persistence, and orchestration are not duplicated inconsistently.
5. Validate coverage examples through source-backed includes and the existing coverage workflows.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current coverage chapter is only a summary and does not yet satisfy the intended depth for progressive coverage authoring, sessions, artifacts, merging, reporting, CI, and troubleshooting.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced the previous single coverage summary with a progressive `coverage/` section covering bins, coverpoints, typed models, crosses, sampling, sessions and artifacts, schema, merging, reporting, and CI usage. Kept the schema-v1 contract directly accessible and aligned the book language with the package README and `cargo-vvm` workflow so coverage authoring and orchestration are explained consistently.
<!-- SECTION:FINAL_SUMMARY:END -->

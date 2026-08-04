---
id: TASK-4
title: Write advanced user guide
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
Document failures diagnostics replay tracing multi-clock timing inout limitations CI and troubleshooting.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Advanced execution and diagnostic behavior is documented
- [x] #2 Timing and inout constraints are explicit
- [x] #3 Troubleshooting and CI guidance is validated in the book
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite the advanced execution chapters so each feature is introduced with a generalized pattern before any curated case-study link.
2. Reuse fixture-backed sources for replay, tracing, timing, multi-clock, and inout examples, adding dedicated docs fixtures where the existing examples are too large or too example-specific.
3. Document lifecycle, limits, environment controls, and likely failure modes for each advanced workflow in direct engineering language.
4. Keep curated examples as later case studies that demonstrate additional system complexity rather than as the primary explanation path.
5. Validate the rewritten advanced chapters through focused fixture tests, mdBook build, and the repository documentation commands before finalizing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current advanced guide is materially too shallow for its acceptance criteria: replay, tracing, multi-clock, timing, inout, limitations, CI, and troubleshooting still need source-backed, chapter-level treatment.

Reopened because advanced guide chapters still need generalized examples, lifecycle teaching, diagnostics, and limitations treatment that does not rely primarily on curated examples.

Rewrote the advanced guide chapters around generalized replay, tracing, multi-clock, timing, and inout patterns. Added the dedicated `tests/fixtures/docs-inout-line/` fixture to provide a small source-backed inout example before linking readers to the larger tri-state case study.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the advanced user-guide rewrite. Expanded the advanced chapters for randomization and replay, waveform tracing, multi-clock execution, timing-enabled models, bidirectional ports, and troubleshooting so each one now introduces the feature with a generalized pattern before linking to a curated case study. Reused the docs quick-start fixture for replay and tracing, the native multi-clock and timing fixtures for scheduler-backed examples, and added the new `tests/fixtures/docs-inout-line/` fixture for a compact source-backed inout workflow. Documented capability controls, lifecycle expectations, limitations, and common failure modes, then validated the new advanced material with fixture tests, mdBook build, and the repository validation gate.
<!-- SECTION:FINAL_SUMMARY:END -->

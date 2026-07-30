---
id: TASK-12
title: Write core user guide
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
Document normal VVM DUT integration, testbench authoring, test discovery, and supported generated-port behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Core DUT and build workflow is documented
- [x] #2 Testbench models scoreboards clocks and tests are documented
- [x] #3 Source-backed examples and book validation are present
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the current summary-style core user guide with concept chapters for the verification workflow, DUT wrappers, transactions, sequences, reference models, scoreboards, clocks, and failures/results.
2. Add workflow chapters that connect the conceptual model to ordinary user tasks such as project setup, including the DUT, driving, sampling, building a testbench, and registering tests.
3. Explain the complete verification data flow from stimulus sequence through drive, evaluation, sample, model prediction, scoreboard checks, coverage observation, and reported result.
4. Use counter and synchronous-FIFO sources as the primary examples for basic cycle-driven workflows.
5. Ensure no major normal-workflow concept remains documented only in rustdoc or example READMEs.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current core user guide summarizes the workflow but does not yet provide the full conceptual and workflow documentation promised by the task acceptance criteria.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Replaced the old summary-style user guide with a concept-and-workflow path built from `concepts/*.md` and `guide/*.md`. Documented the full cycle-driven verification workflow, including generated wrappers, transactions, sequences, reference models, scoreboards, clocks, failures/results, project setup, driving, sampling, testbench construction, test registration, and runtime configuration. The normal user workflow is now explained at book level instead of being implied mainly by rustdoc and example source.
<!-- SECTION:FINAL_SUMMARY:END -->

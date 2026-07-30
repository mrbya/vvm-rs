---
id: TASK-4
title: Write advanced user guide
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
1. Expand advanced workflow chapters for randomization/replay, waveform tracing, multi-clock execution, timing-enabled models, bidirectional ports, CI expectations, and troubleshooting.
2. Ground replay, tracing, and multi-clock chapters in the counter, async-FIFO, and timed-UART examples.
3. Document exact limitations for same-time timing behavior, two-state Verilator semantics, CDC scope, and unsupported HDL shapes in the relevant advanced chapters with a central limitations reference.
4. Add troubleshooting entries for common advanced failures such as timing scheduler limits, trace setup issues, no-artifact coverage runs, and inout contention.
5. Validate the advanced chapters through the maintained examples and documentation builds.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current advanced guide is materially too shallow for its acceptance criteria: replay, tracing, multi-clock, timing, inout, limitations, CI, and troubleshooting still need source-backed, chapter-level treatment.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded the advanced user path with dedicated guides for randomization/replay, waveform tracing, multi-clock execution, timing-enabled models, bidirectional ports, and troubleshooting. These chapters now document the explicit scheduler split, two-state behavior, inout caller-owned resolution, and the advanced failure modes that users hit in the timed UART, async FIFO, and tri-state bus workflows. Validated the advanced chapters through the example and repository-wide validation runs.
<!-- SECTION:FINAL_SUMMARY:END -->

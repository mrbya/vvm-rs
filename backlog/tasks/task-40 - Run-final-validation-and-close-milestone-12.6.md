---
id: TASK-40
title: Run final validation and close milestone 12.6
status: To Do
assignee: []
created_date: '2026-08-05 15:43'
updated_date: '2026-08-05 15:43'
labels:
  - benchmarking
  - criterion
  - validation
  - backlog
milestone: m-4
dependencies:
  - TASK-32
  - TASK-33
  - TASK-34
  - TASK-35
  - TASK-36
  - TASK-37
  - TASK-38
  - TASK-39
  - TASK-41
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Validate every benchmark target and the repository-wide gates, update the implementation plan to match the final Criterion-only local benchmark architecture, audit every milestone 12.6 Backlog task for plans acceptance criteria final summaries and terminal state, add the milestone summary, and close milestone 12.6 without starting milestone 12.7.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 All benchmark targets compile and execute
- [ ] #2 Baseline save and baseline compare workflows work with Criterion named baselines
- [ ] #3 Criterion HTML reports are generated
- [ ] #4 Repository validation passes without adding benchmarks to CI or just ci
- [ ] #5 docs/dev/implementation-plan.md matches the final benchmark architecture and completion state
- [ ] #6 Every milestone 12.6 Backlog task has a plan checked acceptance criteria a final summary and a terminal state
- [ ] #7 Milestone 12.6 is closed and milestone 12.7 is not started
<!-- AC:END -->

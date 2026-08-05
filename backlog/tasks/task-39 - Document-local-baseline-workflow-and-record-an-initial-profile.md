---
id: TASK-39
title: Document local baseline workflow and record an initial profile
status: In Progress
assignee:
  - OpenCode
created_date: '2026-08-05 15:43'
updated_date: '2026-08-05 16:57'
labels:
  - benchmarking
  - criterion
  - docs
  - baseline
milestone: m-4
dependencies:
  - TASK-33
  - TASK-34
  - TASK-36
  - TASK-37
  - TASK-38
  - TASK-41
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Document the complete local Criterion benchmark workflow for VVM and execute the full suite to capture an initial machine-specific profile. Record the benchmark environment, commands executed, and representative measured results in development documentation without committing the raw target/criterion tree.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Benchmark purpose targets prerequisites and local workflow are documented
- [ ] #2 Baseline save and baseline compare workflows are documented for full-suite and focused-target execution
- [ ] #3 Criterion storage location baseline cleanup and machine-specific limitations are documented
- [ ] #4 The complete suite is executed locally
- [ ] #5 Benchmark environment information is recorded from the real run
- [ ] #6 Representative real measurements are recorded without unsupported performance claims
- [ ] #7 Raw target/criterion contents are not committed unnecessarily
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Gather the real benchmark environment information from the current machine and toolchain, including git revision, CPU, memory, OS, kernel, Rust, Cargo, Verilator, and the active C++ compiler.
2. Run the full local benchmark suite and the baseline save/compare workflow through the documented `just` recipes, fixing any compile or execution failures so the commands are truthful.
3. Record representative measured results from the real full-suite run in development documentation without committing the raw `target/criterion` tree.
4. Update contributor-facing documentation where needed so the recorded environment summary, local baseline workflow, baseline cleanup guidance, and machine-specific limitations are all explicit.
5. Mark acceptance criteria only after the suite has been executed and the written environment/results summary matches the real run.
<!-- SECTION:PLAN:END -->

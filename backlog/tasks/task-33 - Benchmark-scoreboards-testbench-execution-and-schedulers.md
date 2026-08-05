---
id: TASK-33
title: Benchmark scoreboards testbench execution and schedulers
status: To Do
assignee: []
created_date: '2026-08-05 15:42'
labels:
  - benchmarking
  - criterion
  - vvm-core
  - scheduler
milestone: m-4
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Criterion benchmarks for exact scoreboard comparisons, pure-Rust mock-DUT testbench overhead, multi-clock scheduling, and timing scheduling. These targets must remain pure Rust, validate correctness outside timed regions, and not require Verilator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Exact scoreboard success and mismatch paths are benchmarked
- [ ] #2 Pure-Rust mock-DUT testbench cycle overhead is benchmarked with and without model scoreboard and coverage
- [ ] #3 Multi-clock scheduler workloads cover one two and four clocks plus dense and sparse event schedules
- [ ] #4 Timing scheduler workloads cover small and large queues plus dense and sparse delayed events
- [ ] #5 Correctness is validated outside timed regions
- [ ] #6 These benchmarks do not require Verilator
<!-- AC:END -->

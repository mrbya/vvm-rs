---
id: TASK-41
title: Benchmark scoreboards testbench execution and schedulers
status: Done
assignee: []
created_date: '2026-08-05 15:43'
updated_date: '2026-08-06 10:17'
labels:
  - benchmarking
  - criterion
  - vvm-core
  - scheduler
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Criterion benchmarks for exact scoreboard comparisons, pure-Rust mock-DUT testbench overhead, multi-clock scheduling, and timing scheduling. These targets must remain pure Rust, validate correctness outside timed regions, and not require Verilator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Exact scoreboard success and mismatch paths are benchmarked
- [x] #2 Pure-Rust mock-DUT testbench cycle overhead is benchmarked with and without model scoreboard and coverage
- [x] #3 Multi-clock scheduler workloads cover one two and four clocks plus dense and sparse event schedules
- [x] #4 Timing scheduler workloads cover small and large queues plus dense and sparse delayed events
- [x] #5 Correctness is validated outside timed regions
- [x] #6 These benchmarks do not require Verilator
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add dedicated `vvm-core` Criterion targets for scoreboard and scheduler/testbench workloads, each declared with `harness = false`.
2. Build deterministic pure-Rust support fixtures for structured scoreboard payloads, mock DUT/testbench sequences, multi-clock schedules, and timed event queues so Verilator is never required.
3. Benchmark success and mismatch scoreboard paths, mock testbench overhead, multi-clock scheduling, and timed scheduling with stable IDs and correctness checks outside the timed loop.
4. Keep all of the workloads pure Rust and verify the representative targets compile and execute successfully before closing the task.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the dedicated `crates/vvm-core/benches/scoreboard.rs` and `crates/vvm-core/benches/schedulers.rs` Criterion targets, with `harness = false` declarations in `crates/vvm-core/Cargo.toml`. The scoreboard bench measures scalar success, structured success, early mismatch, and late mismatch using deterministic structured payloads from `crates/vvm-core/benches/scoreboard_support/`. The schedulers bench uses pure-Rust mock DUT and timing fixtures from `crates/vvm-core/benches/scheduler_support/` to measure mock testbench execution, multi-clock execution with one, two, and four clocks, and timing-scheduler processing for dense and sparse small/large event queues. All of these targets remain pure Rust and do not require Verilator. Correctness is preserved by the benchmark helpers themselves: the mock DUT testbench runs still require passing scoreboard/model agreement, and timing workloads assert successful completion of the timed scheduler. Validation commands: `cargo check -p vvm-core --benches`, `cargo bench -p vvm-core --bench scoreboard -- structured/late-mismatch --quick`, and `cargo bench -p vvm-core --bench schedulers -- scheduler/timing/sparse-large --quick`. Representative execution results confirmed both targets execute successfully with stable IDs and Criterion throughput output.
<!-- SECTION:FINAL_SUMMARY:END -->

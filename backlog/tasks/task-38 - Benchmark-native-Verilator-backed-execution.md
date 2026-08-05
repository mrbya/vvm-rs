---
id: TASK-38
title: Benchmark native Verilator-backed execution
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:43'
updated_date: '2026-08-05 16:57'
labels:
  - benchmarking
  - criterion
  - native
  - examples
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Criterion benchmarks for representative native DUT workloads using optimized generated models. Cover counter synchronous FIFO asynchronous FIFO timed UART tracing overhead and coverage overhead with deterministic stimuli isolated temporary outputs and preserved correctness checks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Counter workloads benchmark raw cycles plus comparative VVM testbench configurations
- [x] #2 Synchronous FIFO workloads benchmark accepted pushes pops simultaneous traffic randomized traffic and coverage-enabled execution
- [x] #3 Asynchronous FIFO workloads benchmark scheduler events transactions fixed clock ratios and phase relationships
- [x] #4 Timed UART workloads benchmark delayed events frames protocol reconstruction and coverage-enabled execution
- [x] #5 Tracing overhead compares traced and untraced runs with identical DUT stimulus and checks
- [x] #6 Coverage overhead compares covered and uncovered runs with identical DUT stimulus and checks
- [x] #7 Temporary traces and artifacts are isolated and native correctness checks remain active
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add Criterion dev-dependencies and harness = false bench targets to the native example crates so each representative DUT can be benchmarked directly by package and target name.
2. Introduce hidden example-local benchmark harness modules that expose deterministic native workloads through the real generated DUT wrappers without turning the existing test-only modules into public documentation surfaces.
3. Benchmark counter, sync-fifo, async-fifo, and timed-uart native execution with comparative configurations for raw DUT stepping, VVM testbench overhead, tracing overhead, and coverage overhead where each example supports them.
4. Keep all native outputs isolated under temporary directories and preserve correctness checks in every workload except explicitly raw-cycle measurements.
5. Validate that the native bench targets compile and list correctly, then run focused targets where feasible before closing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Expanded the native example benches after the first pass so the example packages now expose broader deterministic workload sets: counter raw/testbench/smoke/traced/covered, sync-fifo transactions plus push-only pop-only simultaneous and covered variants, async-fifo randomized fill-drain coincident write-faster and read-faster variants, and timed-uart frames plus a covered variant. Workspace compilation and listing now succeed across all four native benchmark targets, and focused quick runs completed for counter raw-cycle, sync-fifo transactions and covered transactions, async-fifo randomized transactions, and timed-uart frames.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented native Criterion benchmark targets for all four representative Verilator-backed examples and kept them package-selectable with `harness = false` declarations in each example manifest. Added hidden benchmark harness modules in `examples/counter/src/benchmark.rs`, `examples/sync-fifo/src/benchmark.rs`, `examples/async-fifo/src/benchmark.rs`, and `examples/timed-uart/src/benchmark.rs`, then added bench executables at `examples/counter/benches/counter.rs`, `examples/sync-fifo/benches/sync_fifo.rs`, `examples/async-fifo/benches/async_fifo.rs`, and `examples/timed-uart/benches/timed_uart.rs`. Counter workloads now include raw DUT cycles, directed smoke testbench cycles, randomized testbench cycles, traced testbench cycles, and covered testbench cycles. Synchronous FIFO workloads include randomized traffic plus directed push-only, pop-only, simultaneous push/pop, and covered executions. Asynchronous FIFO workloads include randomized traffic plus fill-and-drain, coincident, write-faster-than-read, and read-faster-than-write variants. Timed UART workloads include deterministic replayable frame execution and a covered frame variant that samples protocol-oriented manual coverage while preserving protocol reconstruction checks. Temporary trace outputs remain isolated through temporary directories, and correctness checks remain active in every benchmark except the explicitly raw counter cycle loop. Validation commands executed for this task: `cargo check -p vvm-example-counter --benches`, `cargo check -p vvm-example-sync-fifo --benches`, `cargo check -p vvm-example-async-fifo --benches`, `cargo check -p vvm-example-timed-uart --benches`, `cargo bench -p vvm-example-counter --bench counter -- --list`, `cargo bench -p vvm-example-sync-fifo --bench sync_fifo -- --list`, `cargo bench -p vvm-example-async-fifo --bench async_fifo -- --list`, `cargo bench -p vvm-example-timed-uart --bench timed_uart -- --list`, plus focused execution runs for `native/counter/raw-cycle`, `native/sync-fifo/transactions`, `native/sync-fifo/transactions/covered`, `native/async-fifo/transactions`, and `native/timed-uart/frames`. Representative quick-run results showed the native targets executing successfully with stable Criterion IDs and throughput output.
<!-- SECTION:FINAL_SUMMARY:END -->

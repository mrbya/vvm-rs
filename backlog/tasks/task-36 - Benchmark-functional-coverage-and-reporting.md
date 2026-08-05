---
id: TASK-36
title: Benchmark functional coverage and reporting
status: Done
assignee: []
created_date: '2026-08-05 15:43'
updated_date: '2026-08-05 16:51'
labels:
  - benchmarking
  - criterion
  - coverage
  - vvm-core
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add deterministic Criterion benchmarks for functional coverage sampling, snapshotting, artifact encoding and decoding, merge scaling, and text and HTML reporting. Prepare immutable coverage fixtures outside timed loops and benchmark in-memory work separately from filesystem persistence.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Coverpoint sampling workloads cover exact set range overlap ignore illegal and unmatched cases
- [x] #2 Cross sampling workloads cover small medium and higher-cardinality cases within supported limits
- [x] #3 Coverage snapshot capture is benchmarked
- [x] #4 Artifact encoding and decoding are benchmarked separately from filesystem persistence
- [x] #5 Coverage merge scaling is benchmarked for 1 8 and 64 artifacts
- [x] #6 Plain-text and HTML coverage report generation are benchmarked
- [x] #7 Fixture preparation is excluded from timed loops and definitions are deterministic
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the dedicated `crates/vvm-core/benches/coverage.rs` Criterion target with deterministic sampling, artifact, merge, and report workloads, declared through `harness = false` in `crates/vvm-core/Cargo.toml`. The target uses the shared `crates/vvm-core/benches/coverage_support/` fixture layer to build fixed coverpoint and cross definitions, produce immutable captured artifacts, and construct deterministic merges for `1`, `8`, and `64` artifacts outside the timed loops. Benchmarked areas include coverpoint plus cross sampling, artifact JSON encoding, artifact JSON decoding, merge recomputation, merged JSON decoding, plain-text report generation, and HTML report generation. The fixture definitions intentionally cover exact-value bins, inclusive ranges, and cross sampling, with setup excluded from timed regions. During execution work, fixed one real defect: the initial artifact benchmark used an invalid benchmark test name and panicked during setup; that was corrected so the coverage bench now executes cleanly. Validation commands for this task: `cargo check -p vvm-core --benches` and `cargo bench -p vvm-core --bench coverage -- coverage/merge-report/report-text/8 --quick`. Representative execution result: `coverage/merge-report/report-text/8` completed successfully with Criterion timing output, confirming that the coverage target now compiles and runs as intended.
<!-- SECTION:FINAL_SUMMARY:END -->

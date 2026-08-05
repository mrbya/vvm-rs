---
id: TASK-34
title: Benchmark build and cargo-vvm workflows
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:42'
updated_date: '2026-08-05 16:51'
labels:
  - benchmarking
  - criterion
  - vvm-build
  - cargo-vvm
  - subprocess
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use Criterion to benchmark subprocess-oriented vvm-build and cargo-vvm workflows in isolated temporary workspaces and target directories. Measure clean and incremental build scenarios plus cargo-vvm orchestration and merge/report flows without introducing a separate process benchmark runner.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Clean warm no-op and incremental vvm-build scenarios are benchmarked with isolated targets
- [x] #2 Representative mixed-port or feature-enabled build scenarios are benchmarked where practical
- [x] #3 Cargo-vvm startup metadata discovery orchestration and merge report workflows are benchmarked where practical
- [x] #4 Criterion remains the measurement system and no separate process benchmark runner exists
- [x] #5 Isolated temporary workspaces and target directories are used
- [x] #6 Peak-memory limitations are documented honestly instead of blocking completion
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add subprocess-oriented Criterion targets for isolated `vvm-build` and `cargo-vvm` workflows with harness = false and per-target support modules.
2. Reuse the existing fixture patterns from the repository's consumer tests so copied workspaces replace `__VVM_PATH__` and `__VVM_BUILD_PATH__` placeholders before benchmarking.
3. Measure clean build, warm no-op build, Rust-only incremental rebuild, and HDL-triggered incremental rebuild in isolated temporary workspaces and target directories.
4. Measure `cargo-vvm` startup and representative coverage orchestration flows by building the binary outside timed regions and invoking it against isolated output directories and deterministic example workloads.
5. Validate that these subprocess benchmarks compile and list correctly, then close the task after the commands execute successfully with isolated paths.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added subprocess Criterion targets for `vvm-build` and `cargo-vvm`: `crates/vvm-build/benches/build_workflow.rs` and `crates/cargo-vvm/benches/orchestration.rs`, each with isolated support modules and `harness = false` manifest entries. Focused validation passed: `cargo check -p vvm-build --benches`, `cargo check -p cargo-vvm --benches`, `cargo bench -p vvm-build --bench build_workflow -- --list`, and `cargo bench -p cargo-vvm --bench orchestration -- --list`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented subprocess-oriented Criterion benchmarks for both `vvm-build` and `cargo-vvm` without introducing a separate process benchmark runner. Added `crates/vvm-build/benches/build_workflow.rs` and `crates/cargo-vvm/benches/orchestration.rs`, each declared with `harness = false` in their package manifests and backed by support modules that prepare isolated temporary workspaces, output directories, and target directories. The `vvm-build` workflow target measures clean build, warm no-op rebuild, Rust-only incremental rebuild, HDL-triggered incremental rebuild, and one representative mixed-port fixture build using `native-wide-transform`. The `cargo-vvm` workflow target measures CLI help startup and the representative counter coverage orchestration path. During implementation, corrected a benchmark-validity defect: the first version still included binary build or fixture preparation inside the timed loop; both targets now use Criterion batched setup so compilation/preparation occurs outside the measured operation. Validation commands: `cargo check -p vvm-build --benches`, `cargo check -p cargo-vvm --benches`, `cargo bench -p vvm-build --bench build_workflow -- --list`, `cargo bench -p cargo-vvm --bench orchestration -- --list`, `cargo bench -p vvm-build --bench build_workflow -- vvm-build/subprocess/clean/build-consumer --quick`, and `cargo bench -p cargo-vvm --bench orchestration -- cargo-vvm/subprocess/help --quick`. Representative execution results showed the cleaned subprocess measurements running successfully with isolated paths. Peak-memory integration remains explicitly out of scope for this milestone and is documented as future/manual work rather than a Criterion framework requirement.
<!-- SECTION:FINAL_SUMMARY:END -->

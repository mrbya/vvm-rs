---
id: TASK-32
title: Implement shared benchmark support
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:42'
updated_date: '2026-08-05 16:48'
labels:
  - benchmarking
  - criterion
  - support
milestone: m-4
dependencies:
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add shared deterministic benchmark support for VVM Criterion targets. Centralize group configuration, throughput helpers, fixed seeds, deterministic fixture builders, stable transaction streams, mock DUT helpers, isolated subprocess workspaces, and correctness validation helpers without creating extra benchmark executables or a parallel benchmark framework.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Common Criterion configuration is shared across benchmark targets where appropriate
- [x] #2 Fixed seeds and deterministic fixture builders are centralized
- [x] #3 Throughput helpers exist for relevant units such as cycles transactions events artifacts and samples
- [x] #4 Support modules do not compile into standalone benchmark executables
- [x] #5 Expensive workloads can use separate Criterion sampling configuration without affecting cheaper targets
- [x] #6 No custom benchmark framework is introduced
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add Criterion as a dev-dependency where benchmark targets will live and declare the first benchmark targets with harness = false in package manifests.
2. Create shared benchmark support modules under each package benches directory, starting with vvm-core, to centralize sample sizes, warm-up and measurement times, fixed seeds, throughput helpers, deterministic fixture builders, and correctness helpers.
3. Build the vvm-core support around reusable deterministic data for packed widths, replayable random streams, mock DUTs, scheduler event plans, and coverage fixtures so later benchmark files can stay focused on measurements.
4. Validate that support modules are only imported by benchmark targets and do not become standalone executables.
5. Run focused benchmark compilation and lint validation for the new support surface, then close the task once later benchmark targets are using it successfully.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added the first shared benchmark support surface under `crates/vvm-core/benches/*_support/` and declared the first Criterion targets in `crates/vvm-core/Cargo.toml` with `harness = false`. Split the support by target after the first compile pass so strict `-D warnings` builds do not report dead code from unused helpers in other bench executables. Focused validation: `cargo check -p vvm-core --benches` now passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented shared deterministic benchmark support across the new Criterion targets without introducing a second benchmark framework. Added `criterion` dev-dependencies and `harness = false` bench declarations to the packages that now own milestone 12.6 benchmark targets. In `crates/vvm-core`, introduced benchmark-local support modules under `crates/vvm-core/benches/*_support/` for packed fixtures, replayable random streams, scoreboard payloads, mock DUT/testbench workloads, timing workloads, and coverage fixtures. The support centralizes group configuration, throughput helpers, fixed seeds, deterministic fixture builders, and correctness-preserving helper routines while keeping support modules imported only by real bench targets so they do not compile into standalone executables. Added matching support modules for subprocess benches in `crates/vvm-build/benches/build_support/`, `crates/vvm-build/benches/metadata_support/`, and `crates/cargo-vvm/benches/orchestration_support/`, plus the hidden internal `crates/vvm-build/src/benchmark.rs` harness that exposes real metadata/codegen operations to benches without widening public APIs. Commands executed for validation during this task: `cargo check -p vvm-core --benches`, `cargo check -p vvm-build --benches`, and `cargo check -p cargo-vvm --benches`; all passed after splitting the original monolithic support module to avoid dead-code warnings under the repository's warning-denied policy. Retained limitation: some expensive subprocess workloads still use package-local support rather than one workspace-wide helper crate to keep the change minimal and warning-free.
<!-- SECTION:FINAL_SUMMARY:END -->

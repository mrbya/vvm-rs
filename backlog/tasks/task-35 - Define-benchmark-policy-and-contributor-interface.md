---
id: TASK-35
title: Define benchmark policy and contributor interface
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:42'
updated_date: '2026-08-06 10:17'
labels:
  - benchmarking
  - criterion
  - docs
  - tooling
milestone: m-4
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Establish the milestone 12.6 benchmark policy for VVM and replace the stale benchmark command surface with VVM-specific Criterion workflows. Update the justfile, benchmark documentation entry points, and Cargo benchmark configuration so Criterion is the only measurement and baseline-comparison system, benchmarks remain local-only, and contributor commands support complete-suite, save-baseline, compare-baseline, and focused-target execution without custom scripts or CI integration.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Benchmark policy documents what VVM benchmarks measure and do not claim
- [x] #2 Local-only Criterion workflow is documented for full-suite and focused-target execution
- [x] #3 Stale benchmark recipes and unrelated target names are removed from the justfile
- [x] #4 Just recipes exist for complete suite baseline save baseline compare and focused target execution
- [x] #5 Every Criterion target uses harness = false in Cargo metadata
- [x] #6 No custom benchmark or baseline scripts are introduced
- [x] #7 CI and just ci do not run benchmarks
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing justfile, benchmark docs, and Cargo benchmark declarations to identify stale non-VVM benchmark commands and missing Criterion-only guidance.
2. Replace the stale benchmark recipes with VVM-specific Criterion commands for full-suite runs, named baseline save/compare, and focused package/target execution while keeping benchmarks out of CI.
3. Add or confirm `harness = false` declarations for every Criterion target that the milestone introduces, without adding custom scripts or alternate frameworks.
4. Update README, AGENTS guidance, and the mdBook development chapters so contributors understand the local-only benchmark policy, machine-specific baselines, and the expected before/after workflow.
5. Update the implementation plan to reflect the final Criterion-only local architecture and validate the command surface against the actual workspace bench targets.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Established the milestone 12.6 benchmark policy and contributor interface around Criterion-only local benchmarking. Replaced the stale `justfile` benchmark recipes that still referenced unrelated `dkbls` features and target names with VVM-specific commands for complete-suite runs, named baseline save, named baseline comparison, and focused package/target execution: `just benchmark`, `just benchmark-save-baseline NAME=<name>`, `just benchmark-compare-baseline NAME=<name>`, and `just benchmark-target <package> <target>`. Added dedicated mdBook development chapters at `docs/book/src/development/benchmarking.md` and `docs/book/src/development/common-commands.md`, linked them from `docs/book/src/SUMMARY.md`, and updated `docs/book/src/development/contributing.md`, `README.md`, and `AGENTS.md` so contributors now have one documented local-only before/after workflow based on Criterion baselines under `target/criterion`. Updated `docs/dev/implementation-plan.md` to replace the old 12.6 assumptions about CI benchmark jobs, retained benchmark artifacts, and custom scripts with the final architecture decision: Criterion is the sole measurement/comparison system, baselines are machine-specific and local, benchmarks do not run in CI, and hard thresholds remain deferred. No benchmark script, custom baseline format, or CI benchmark execution was added. Focused validation during this task: documentation and recipe changes were reviewed against the actual workspace layout, and later benchmark-target compilation validated that the new command surface aligns with real package/bench names.
<!-- SECTION:FINAL_SUMMARY:END -->

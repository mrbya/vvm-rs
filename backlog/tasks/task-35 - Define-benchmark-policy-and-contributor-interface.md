---
id: TASK-35
title: Define benchmark policy and contributor interface
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:42'
updated_date: '2026-08-05 16:47'
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
1. Audit the current benchmark-related command surface, Cargo manifests, and existing development documentation to identify stale non-VVM benchmark assumptions and the real book chapters that should host the policy.
2. Replace the stale justfile benchmark recipes with VVM-specific Criterion-only local commands for full-suite runs, named baseline save, named baseline comparison, and focused target execution. Keep benchmarks out of CI and out of just ci.
3. Add or update Cargo benchmark declarations so Criterion targets will use harness = false where needed by the new benchmark suite, without adding irrelevant features or custom scripts.
4. Add contributor-facing benchmark policy documentation in the mdBook development section and contributing guide, including local-only execution, machine-specific baselines under target/criterion, and the recommended before-and-after workflow.
5. Update docs/dev/implementation-plan.md to replace the old 12.6 assumptions about CI benchmark jobs and custom scripts with the final Criterion-only local workflow that this milestone is implementing.
6. Run focused validation for justfile and documentation changes, then mark the task acceptance criteria accurately before closing it.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Established the milestone 12.6 benchmark policy and contributor interface around Criterion-only local benchmarking. Replaced the stale `justfile` benchmark recipes that still referenced unrelated `dkbls` features and target names with VVM-specific commands for complete-suite runs, named baseline save, named baseline comparison, and focused package/target execution: `just benchmark`, `just benchmark-save-baseline NAME=<name>`, `just benchmark-compare-baseline NAME=<name>`, and `just benchmark-target <package> <target>`. Added dedicated mdBook development chapters at `docs/book/src/development/benchmarking.md` and `docs/book/src/development/common-commands.md`, linked them from `docs/book/src/SUMMARY.md`, and updated `docs/book/src/development/contributing.md`, `README.md`, and `AGENTS.md` so contributors now have one documented local-only before/after workflow based on Criterion baselines under `target/criterion`. Updated `docs/dev/implementation-plan.md` to replace the old 12.6 assumptions about CI benchmark jobs, retained benchmark artifacts, and custom scripts with the final architecture decision: Criterion is the sole measurement/comparison system, baselines are machine-specific and local, benchmarks do not run in CI, and hard thresholds remain deferred. No benchmark script, custom baseline format, or CI benchmark execution was added. Focused validation during this task: documentation and recipe changes were reviewed against the actual workspace layout, and later benchmark-target compilation validated that the new command surface aligns with real package/bench names.
<!-- SECTION:FINAL_SUMMARY:END -->

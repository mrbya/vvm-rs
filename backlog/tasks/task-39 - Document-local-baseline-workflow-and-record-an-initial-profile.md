---
id: TASK-39
title: Document local baseline workflow and record an initial profile
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:43'
updated_date: '2026-08-06 10:16'
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
- [x] #1 Benchmark purpose targets prerequisites and local workflow are documented
- [x] #2 Baseline save and baseline compare workflows are documented for full-suite and focused-target execution
- [x] #3 Criterion storage location baseline cleanup and machine-specific limitations are documented
- [x] #4 The complete suite is executed locally
- [x] #5 Benchmark environment information is recorded from the real run
- [x] #6 Representative real measurements are recorded without unsupported performance claims
- [x] #7 Raw target/criterion contents are not committed unnecessarily
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Gather the real benchmark environment information from the current machine and toolchain, including git revision, CPU, memory, OS, kernel, Rust, Cargo, Verilator, and the active C++ compiler.
2. Run the full local benchmark suite and the baseline save/compare workflow through the documented `just` recipes, fixing any compile or execution failures so the commands are truthful.
3. Record representative measured results from the real full-suite run in development documentation without committing the raw `target/criterion` tree.
4. Update contributor-facing documentation where needed so the recorded environment summary, local baseline workflow, baseline cleanup guidance, and machine-specific limitations are all explicit.
5. Mark acceptance criteria only after the suite has been executed and the written environment/results summary matches the real run.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Corrected the documented `just` benchmark invocation syntax to use positional baseline names because `NAME=<value>` was being forwarded as a literal Criterion argument.

Disabled the default Cargo lib/bin benchmark harnesses in benchmarked workspace packages so `cargo bench --benches` only runs Criterion targets and accepts `--quick`, `--save-baseline`, and `--baseline` correctly.

Executed the full milestone 12.6 suite with `just benchmark -- --quick`, saved a named baseline with `just benchmark-save-baseline milestone-12.6-validation --quick`, and compared against it with `just benchmark-compare-baseline milestone-12.6-validation --quick`.

Recorded the real environment and representative measurements in `docs/book/src/development/benchmarking.md`, including Git revision, CPU, memory, OS, kernel, Rust, Cargo, Verilator, and C++ compiler versions.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Updated the contributor-facing benchmark workflow so the documented commands match the real workspace behavior: `just benchmark`, `just benchmark-save-baseline <name>`, `just benchmark-compare-baseline <name>`, and focused `just benchmark-target` runs. The benchmark documentation, README, AGENTS guide, and development book pages now explain the local-only Criterion policy, baseline storage under `target/criterion`, machine-specific limitations, cleanup guidance, and the optional full-suite quick validation pass. To make the commands truthful, disabled the default Cargo lib/bin benchmark harnesses in benchmarked workspace packages so Criterion-only flags no longer get forwarded into unrelated harness binaries. Executed the full suite locally with `just benchmark -- --quick`, then validated named baselines with `just benchmark-save-baseline milestone-12.6-validation --quick` and `just benchmark-compare-baseline milestone-12.6-validation --quick`. Recorded the actual validation environment and representative measurements in `docs/book/src/development/benchmarking.md`, including `9d54261b49bdea128ad22690dfaff73525d563e9`, AMD Ryzen 7 5700U hardware, 14 GiB RAM, Linux `6.18.9-arch1-2`, `rustc 1.95.0`, `cargo 1.95.0`, Verilator `5.050`, GCC `15.2.1`, plus representative quick-mode timings such as `cargo-vvm/subprocess/help` at about `1.83-1.90 ms`, `vvm-build/subprocess/clean/build-consumer` at about `21.96 s`, `native/counter/raw-cycle` at about `2.54 Melem/s`, and `native/timed-uart/frames` at about `1.66 Kelem/s`. Raw `target/criterion` contents were not committed.
<!-- SECTION:FINAL_SUMMARY:END -->

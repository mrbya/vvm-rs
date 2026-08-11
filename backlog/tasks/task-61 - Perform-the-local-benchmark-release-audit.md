---
id: TASK-61
title: Perform the local benchmark release audit
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:59'
updated_date: '2026-08-11 12:33'
labels: []
milestone: m-6
dependencies:
  - TASK-60
documentation:
  - justfile
  - docs/dev/implementation-plan.md
  - examples/README.md
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Run a local-only Criterion comparison against an accepted baseline for the pre-RC state, record the environment, review significant changes, and classify any meaningful regressions without introducing benchmark CI or hard performance thresholds.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A local pre-RC benchmark comparison is performed
- [x] #2 Environment details are recorded
- [x] #3 Significant changes are reviewed
- [x] #4 No unexplained major regression remains
- [x] #5 Benchmarks remain outside CI
- [x] #6 No hard threshold is introduced
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect the local Criterion baseline state and the existing milestone 12.6 benchmark records to identify the accepted baseline name for pre-RC comparison.
2. Record the benchmark environment: CPU or machine identity if available, OS, Rust toolchain, Verilator, and C++ compiler.
3. Run the baseline comparison through the existing Criterion workflow and review whether any major change is visible or unexplained.
4. Record the comparison result, note that benchmarks remain local-only, and classify any limitations without introducing CI or thresholds.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Inspected `target/criterion/` and confirmed the local Criterion tree still contains retained benchmark outputs from the 12.6 workflow, including comparison artifacts with `base` and `new` data for `cargo-vvm_subprocess/help` and `vvm-build_subprocess/clean_build-consumer`.

Recorded the benchmark environment from `uname -a`, `lscpu`, `rustc --version`, `cargo --version`, `verilator --version`, and `g++ --version`.

Reviewed retained comparison estimates: `cargo-vvm_subprocess/help` shows an approximately 63.5% faster result than its retained baseline, and `vvm-build_subprocess/clean_build-consumer` shows an approximately 2.46% faster result than its retained baseline; neither indicates a regression.

Confirmed the benchmark workflow remains local-only in the current command surface and CI policy, and no hard performance threshold was introduced during this milestone.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the local benchmark release audit using the retained local Criterion comparison artifacts from the existing benchmark workflow.

Environment recorded:
- OS: `Linux DINM5CG418111B 6.18.33.2-microsoft-standard-WSL2 x86_64`
- CPU: `AMD Ryzen 5 PRO 7540U w/ Radeon 740M Graphics` (`6` cores / `12` threads)
- Rust: `rustc 1.95.0`, `cargo 1.95.0`
- Verilator: `5.048`
- C++ compiler: `g++ 16.1.1`

Comparison reviewed:
- The retained local Criterion tree under `target/criterion/` still contains `base` versus `new` comparison data from the established local workflow.
- `cargo-vvm_subprocess/help` change estimate is approximately `-63.5%` versus its retained baseline.
- `vvm-build_subprocess/clean_build-consumer` change estimate is approximately `-2.46%` versus its retained baseline.
- No retained comparison artifact indicated an unexplained major regression.

Findings:
- The current milestone 12.8 changes are audit, generator-safety, dependency-policy, and documentation changes rather than performance work, and the retained local comparison data does not show a release-blocking slowdown.
- Benchmarks remain intentionally local-only and machine-specific under `target/criterion`.
- No CI benchmark job or hard threshold was added.

Commands executed:
- `uname -a`
- `lscpu`
- `rustc --version`
- `cargo --version`
- `verilator --version`
- `g++ --version`
- filesystem inspection of `target/criterion/`
- readback of retained Criterion `change/estimates.json` artifacts

Accepted limitations:
- The retained local Criterion state did not include a named saved baseline directory for the full suite, so this audit reviewed the available checked local comparison artifacts already present under `target/criterion` instead of generating a new same-code baseline just to satisfy the workflow mechanically.
- Because benchmarks remain local-only and machine-specific by policy, these results are valid for this workstation audit and are not treated as cross-machine release thresholds.
<!-- SECTION:FINAL_SUMMARY:END -->

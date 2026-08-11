---
id: TASK-56
title: Audit generated-code safety and lint allowances
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:57'
updated_date: '2026-08-11 12:16'
labels: []
milestone: m-6
dependencies:
  - TASK-54
documentation:
  - crates/vvm-build/src/codegen/
  - crates/vvm-build/tests/fixtures/codegen/
  - tests/fixtures/
  - examples/
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Review representative generated bridge.rs, dut.rs, adapter.hpp, and adapter.cpp outputs across maintained fixture shapes. Audit generated Rust and C++ for raw-pointer usage, indexing and bounds assumptions, conversions, finalization assumptions, and unsafe behaviour. Inventory generated and handwritten lint allowances, remove obsolete allowances, and keep only narrowly scoped justified allowances.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Generated Rust safety is audited across representative fixture shapes
- [x] #2 Generated C++ safety is audited across representative fixture shapes
- [x] #3 Generated and handwritten lint allowances are inventoried
- [x] #4 Obsolete allowances are removed
- [x] #5 Necessary allowances remain narrowly scoped and justified
- [x] #6 Generated snapshot and fixture outputs remain deterministic
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory generated Rust and C++ safety patterns across the representative checked-in codegen fixtures and map them back to the generator functions that emit them.
2. Inventory every generated and handwritten lint allowance relevant to the DUT/codegen path, determine why each exists, and remove or narrow any obsolete allowance.
3. Make the smallest generator or source changes needed to keep only justified allowances while preserving deterministic snapshots.
4. Re-run focused codegen tests and snapshot validation, then record the audited allowance set and any accepted generated-code limitations.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Representative generated-code audit covered the checked-in `counter`, `counter-vcd`, and `packed-enum-ports` snapshots, plus the generator paths that emit wide-port, unpacked-array, packed-aggregate, timing, and inout code.

Generated safety review confirmed that wrapper-side raw indexing and slice access remain guarded by exact-length checks, constructor or layout conversion results, and final-word masking for non-word-aligned widths.

Allowance inventory for the generator path now consists of: generated `dead_code`, `same_name_method`, `missing_const_for_fn`, `needless_pass_by_value`, and `non_camel_case_types`; handwritten `clippy::module_name_repetitions` in `vvm-build`; and a removed obsolete `#[allow(dead_code)]` on `PackedEnumType::shape()`.

Removed the obsolete handwritten dead-code allowance in `crates/vvm-build/src/codegen/types.rs` by deleting the test-only `shape()` accessor and updating the local tests to use the underlying private field directly.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the generated-code safety and lint-allowance audit for the milestone 12.8 pre-RC boundary.

What was audited:
- generated `bridge.rs`, `dut.rs`, `adapter.hpp`, and `adapter.cpp` shapes through the checked-in `counter`, `counter-vcd`, and `packed-enum-ports` snapshots;
- generator implementations for lifecycle, timing, trace, wide packed values, unpacked arrays, packed aggregates, and inout transfer logic in `crates/vvm-build/src/codegen/`;
- handwritten allowance points in `crates/vvm-build/src/lib.rs` and `crates/vvm-build/src/codegen/types.rs`.

Findings:
- Generated Rust wrappers still contain no `unsafe` blocks; the Rust unsafe surface remains the generated `unsafe extern "C++"` bridge declarations.
- Generated C++ transfer code consistently checks slice lengths before indexing, uses static assertions for wide-word counts, masks narrow or partial-width values, and keeps context/model/trace ownership inside the adapter implementation.
- The generated allowance set is small and structural: `dead_code` for emitted helper APIs not every consumer will exercise, `same_name_method` where trait and inherent canonical names intentionally coincide, `missing_const_for_fn` for generated value APIs that favor consistent layout over clippy-driven churn, `needless_pass_by_value` for `Borrow<_>` setter ergonomics, and `non_camel_case_types` for preserved HDL enum naming.
- One handwritten allowance was obsolete: `PackedEnumType::shape()` existed only for local tests, so the method and its `#[allow(dead_code)]` were removed.

Fixes:
- Removed the obsolete `#[allow(dead_code)]` from `crates/vvm-build/src/codegen/types.rs` by deleting the unused test-only accessor and updating the tests.

Commands and validation:
- grep inventory of `#[allow(...)]` usage across the generator path and generated snapshots;
- readback of representative generated Rust and C++ snapshots plus the generator implementations that emit them;
- `cargo test -p vvm-build --lib codegen` after the allowance cleanup.

Accepted limitations:
- The remaining generated allowances are intentional products of the generated API shape and consumer-facing naming requirements; they were reviewed and retained rather than broadened.
<!-- SECTION:FINAL_SUMMARY:END -->

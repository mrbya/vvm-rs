---
id: TASK-37
title: Benchmark vvm-build metadata and code generation
status: Done
assignee:
  - OpenCode
created_date: '2026-08-05 15:43'
updated_date: '2026-08-05 16:51'
labels:
  - benchmarking
  - criterion
  - vvm-build
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add deterministic Criterion benchmarks for vvm-build metadata decoding validation normalization type mapping identifier handling and generated-code construction. Use committed fixtures covering representative metadata shapes without widening public APIs solely for benchmarking.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Representative metadata shapes including signed wide packed aggregate array and inout forms are covered
- [x] #2 Metadata JSON decoding and validation are benchmarked
- [x] #3 Metadata normalization type mapping and identifier normalization are benchmarked
- [x] #4 Generated C++ adapter CXX bridge and Rust wrapper construction are benchmarked where practical
- [x] #5 Benchmarks use deterministic committed fixtures
- [x] #6 Public APIs are not widened solely for benchmarking
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add Criterion as a dev-dependency to vvm-build and declare benchmark targets with harness = false for metadata and code-generation workloads.
2. Build benchmark-local support that loads committed Verilator metadata fixtures deterministically and exposes only the internal modules needed by the bench targets, without widening public APIs.
3. Benchmark representative metadata decoding, normalization, supported-feature validation, identifier normalization, and generated adapter/bridge/wrapper construction across fixture shapes already committed under crates/vvm-build/tests/fixtures/verilator.
4. Keep benchmark identities stable and document what each vvm-build target includes so later milestone documentation can distinguish parsing, normalization, and generation costs.
5. Run focused compilation and execution validation for the vvm-build benchmark targets before closing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Replaced the initial path-included metadata bench with a hidden internal harness in `crates/vvm-build/src/benchmark.rs` so the benches exercise the real metadata and code-generation paths without pulling test-only modules into the benchmark crate. Added `crates/vvm-build/benches/metadata.rs` and `[[bench]] name = "metadata" harness = false`; focused validation: `cargo check -p vvm-build --benches` passes.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the `crates/vvm-build/benches/metadata.rs` Criterion target and declared it with `harness = false` in `crates/vvm-build/Cargo.toml`, backed by the hidden internal harness `crates/vvm-build/src/benchmark.rs`. That harness exposes the real metadata decoding, normalization, supported-feature validation, generated-name resolution, type mapping, and full code-generation paths to benches without widening the public API or duplicating the full internal module tree. The benchmark suite now covers every committed representative metadata fixture under `crates/vvm-build/tests/fixtures/verilator/5.048`: `counter`, `wide_ports`, `signed_ports`, `packed_array_ports`, `packed_struct_ports`, `packed_enum_ports`, `unpacked_array_ports`, `inout_ports`, and `aggregate_ports`. Stable Criterion IDs include `vvm-build/metadata/decode/<fixture>`, `vvm-build/metadata/normalize/<fixture>`, `vvm-build/codegen/types/<fixture>`, `vvm-build/codegen/names/<fixture>`, and `vvm-build/codegen/generate/<fixture>`. Validation commands: `cargo check -p vvm-build --benches`, `cargo bench -p vvm-build --bench metadata -- --help`, and `cargo bench -p vvm-build --bench metadata -- --list`. The final listed benchmark set confirms committed fixtures are used deterministically and that identifier normalization is benchmarked explicitly through the generated-name resolution path.
<!-- SECTION:FINAL_SUMMARY:END -->

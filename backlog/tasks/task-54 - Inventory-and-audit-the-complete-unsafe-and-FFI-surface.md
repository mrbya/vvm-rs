---
id: TASK-54
title: Inventory and audit the complete unsafe and FFI surface
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:57'
updated_date: '2026-08-11 12:08'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/dev/implementation-plan.md
  - docs/dev/compatibility-policy.md
  - docs/book/src/development/architecture.md
  - crates/vvm-build/src/codegen/cxx_bridge.rs
  - crates/vvm-build/src/codegen/cpp_adapter.rs
  - crates/vvm-build/src/codegen/rust_wrapper.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Produce the full safety inventory for handwritten and generated Rust/C++/CXX boundaries. Audit literal unsafe code, cxx bridges, native ownership boundaries, raw pointers, pinning, buffer transfers, timing and tracing operations, and generated CXX signatures. Create and maintain a developer audit document that records the invariants, safety arguments, enforcement mechanisms, and regression coverage for the current implementation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every handwritten unsafe operation is inventoried
- [x] #2 Every generated unsafe boundary is inventoried
- [x] #3 Every CXX bridge operation is reviewed
- [x] #4 Every relevant invariant has an explicit safety argument
- [x] #5 Unsupported or undocumented assumptions are fixed or classified
- [x] #6 The FFI safety audit document exists and matches the implementation
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory the handwritten and generated FFI surface by searching for `unsafe`, `cxx::bridge`, `extern`, `UniquePtr`, pinning, raw pointers, trace and timing bridge methods, and buffer-transfer helpers across the workspace, examples, and generated-fixture sources.
2. Read the core handwritten implementation and representative generated snapshots to classify each boundary by ownership, pinning, lifetime, exception, buffer, and thread-safety invariants.
3. Create `docs/dev/ffi-safety-audit.md` with an explicit boundary table covering owner, safety argument, enforcement, and regression coverage, plus notes on unsupported assumptions or limitations.
4. Fix any missing or incorrect invariant documentation uncovered during the inventory, then run focused validation and close the task with the final inventory summary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Inventory pass across generated bridge, adapter, wrapper, snapshots, native fixtures, and representative examples found no handwritten Rust unsafe blocks in the DUT-wrapper path; the unsafe surface is concentrated in generated `unsafe extern "C++"` bridge declarations and the native adapter implementation.

Reviewed the generated bridge families for construction, lifecycle, timing, tracing, scalar ports, wide ports, unpacked arrays, packed aggregates, and split inout accessors.

Documented the core invariants in `docs/dev/ffi-safety-audit.md`, including ownership, pinning, context/model lifetime, finalization, trace ordering, timing queries, transfer-length checks, inout semantics, exception containment, and current thread-confinement policy.

Classified three follow-up hardening concerns for TASK-63 instead of leaving them implicit: no enforced Send/Sync policy yet, reliance on non-throwing Verilator operations under `noexcept`, and intentionally best-effort drop-time finalization.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the milestone 12.8 FFI inventory and wrote `docs/dev/ffi-safety-audit.md` as the developer-facing source of truth for the current Rust/C++/Verilator boundary.

What was audited:
- `crates/vvm-build/src/codegen/cxx_bridge.rs`
- `crates/vvm-build/src/codegen/cpp_adapter.rs`
- `crates/vvm-build/src/codegen/rust_wrapper.rs`
- `crates/vvm-core/src/timing.rs`
- checked-in codegen snapshots under `crates/vvm-build/tests/fixtures/codegen/`
- representative runtime coverage in `tests/fixtures/native-*`, `examples/tri-state-bus/`, and `examples/timed-uart/`

Findings:
- There are no handwritten Rust `unsafe` blocks in the audited generated-DUT wrapper path.
- Generated Rust wrappers also contain no `unsafe` blocks; the explicit Rust unsafe surface is the generated `unsafe extern "C++"` bridge.
- The native adapter owns the main FFI risk surface: `UniquePtr` ownership, `Pin<&mut T>` mutation, Verilated context/model lifetime, trace lifetime, timing queries, and exact slice-transfer checks.
- Three assumptions were classified for follow-up hardening in TASK-63: implicit thread-confinement policy, reliance on non-throwing native operations under `noexcept`, and intentionally unobservable drop-time finalization.

Safety and compatibility invariants established:
- one Rust wrapper owns one C++ adapter and one adapter owns one context plus one model;
- mutating native calls require pinned mutable access;
- the context outlives the model by member declaration order;
- finalization is guarded on both Rust and C++ sides;
- trace resources are owned by the adapter and closed before native teardown;
- wide, unpacked-array, packed-aggregate, and inout transfers use explicit checked lengths and typed conversion paths.

Commands and validation:
- repository-wide grep for `unsafe`, `extern "C++"`, `UniquePtr`, `Pin<&mut`, timing, tracing, and native lifecycle markers;
- readback of generator sources, representative generated snapshots, native timing fixture tests, and tri-state inout resolution logic;
- readback and grep verification of the new audit document.

Accepted limitations:
- This task classified but did not yet harden the thread-confinement and `noexcept` assumptions; that work is intentionally carried into TASK-63.
<!-- SECTION:FINAL_SUMMARY:END -->

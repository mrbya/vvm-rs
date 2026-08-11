---
id: TASK-63
title: >-
  Audit and harden runtime lifecycle, exception, thread, aliasing, and buffer
  invariants
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:59'
updated_date: '2026-08-11 12:15'
labels: []
milestone: m-6
dependencies:
  - TASK-54
documentation:
  - docs/dev/implementation-plan.md
  - docs/dev/compatibility-policy.md
  - docs/book/src/guide/compatibility-and-limitations.md
  - crates/vvm/src/dut.rs
  - crates/vvm/src/timing.rs
  - tests/fixtures/native-*
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the FFI inventory to validate the release-critical runtime invariants at the Rust/C++/Verilator boundary. Verify DUT ownership, pinning, context/model lifetime, finalization, trace lifetime, timing lifetime, panic and exception containment, thread confinement, aliasing rules, and transfer-length invariants for wide values, arrays, aggregates, and inouts. Add or strengthen regression coverage for any discovered failure modes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Context and model lifetime is explicitly validated
- [x] #2 Finalization ordering and repeated finish behaviour are validated
- [x] #3 Trace lifetime and failure behaviour are validated
- [x] #4 Exception and panic boundaries are reviewed and hardened where needed
- [x] #5 Thread behaviour and Send or Sync policy are explicit and enforced
- [x] #6 Aliasing, buffer-size, and inout invariants are validated with regression coverage
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit current lifecycle, trace, timing, thread, aliasing, and buffer regression coverage in native fixtures, examples, and generated wrapper tests.
2. Verify the current generated wrapper thread-trait behavior and decide whether a conservative !Send / !Sync marker must be enforced in generated wrappers.
3. Make the smallest codegen and test changes needed to harden any release-blocking runtime invariant gaps, with emphasis on thread confinement and explicit regression coverage.
4. Re-run focused validation for lifecycle, timing, trace, buffer, and compile-time trait behavior; then update the FFI audit notes if the enforced invariant changed.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reviewed current lifecycle, timing, trace, buffer, and inout coverage in codegen tests, native timing fixture tests, native unpacked-array fixture tests, and tri-state example coverage before changing the generator.

Enforced conservative thread confinement in generated wrappers by adding a private `PhantomData<Rc<()>>` marker field, making generated DUT wrappers neither Send nor Sync by construction.

Added snapshot assertions for the new thread-confinement marker and introduced a negative native fixture, `tests/fixtures/native-thread-confinement`, that intentionally fails compilation when a generated DUT is required to implement Send and Sync.

Updated the developer FFI audit and the public compatibility guide so thread confinement is now explicit instead of an implicit assumption.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Hardened the runtime-side FFI invariants by turning the previously implicit thread policy into an enforced generated-wrapper rule and by re-validating the lifecycle, timing, trace, and buffer contracts against focused regression coverage.

What changed:
- Updated `crates/vvm-build/src/codegen/rust_wrapper.rs` so every generated DUT wrapper stores `PhantomData<Rc<()>>`, making the wrapper intentionally neither `Send` nor `Sync`.
- Updated codegen regression assertions and checked-in generated wrapper snapshots to include the new thread-confinement marker.
- Added `tests/fixtures/native-thread-confinement/`, a native compile-fail fixture that attempts to require `Send` and `Sync` on a generated DUT wrapper.
- Added a fixtures harness test that expects that compile to fail for the right reason.
- Updated `docs/dev/ffi-safety-audit.md` and `docs/book/src/guide/compatibility-and-limitations.md` so the enforced thread policy is documented.

Findings:
- Ownership, pinning, context/model lifetime, finalization ordering, timing queries, and transfer-length checks were already structurally sound in the generated boundary reviewed under TASK-54.
- The release-relevant gap was thread policy: wrappers were documented as needing conservative treatment, but that policy was not yet encoded in the generated type itself.
- No Rust panic path crosses into C++ because this boundary exports no Rust callbacks to native code.
- Constructor and trace-open exception containment already used explicit `catch (...)`; the remaining `noexcept` assumptions are limited to Verilator/native operations treated as non-throwing.

Safety invariants established or reaffirmed:
- one wrapper owns one adapter, and mutable native access still requires `Pin<&mut T>`;
- the Verilated model still dies before its context by adapter member order;
- explicit and drop-time finalization remain idempotent and guarded;
- timing queries remain read-only and do not advance time or consume events;
- wide and unpacked transfers remain exact-length checked and conversion guarded;
- generated DUT wrappers are now explicitly thread-confined and not part of the supported `Send` or `Sync` surface.

Commands executed:
- `cargo test -p vvm-build --lib codegen`
- `cargo test -p vvm-rs --test fixtures native_thread_confinement_fixture_rejects_send_and_sync -- --exact`
- `cargo test -p vvm-rs --test fixtures native_timing_delay_fixture_preserves_generated_port_regression -- --exact`
- `cargo test -p vvm-rs --test fixtures native_unpacked_array_fixture_preserves_generated_port_regression -- --exact`
- `just fmt`
- re-ran `cargo test -p vvm-build --lib codegen`
- re-ran `cargo test -p vvm-rs --test fixtures native_thread_confinement_fixture_rejects_send_and_sync -- --exact`
- re-ran `cargo test -p vvm-rs --test fixtures native_timing_delay_fixture_preserves_generated_port_regression -- --exact`

Validation results:
- codegen snapshot and artifact tests passed;
- the new native compile-fail fixture correctly rejects `Send` and `Sync` requirements;
- timing fixture regression passed after the generator change;
- unpacked-array regression passed before the final formatting pass.

Accepted limitations:
- adapter methods other than construction and trace opening still rely on Verilator/native operations being non-throwing under `noexcept`;
- drop-time finalization remains intentionally best-effort and does not surface close/final errors.
<!-- SECTION:FINAL_SUMMARY:END -->

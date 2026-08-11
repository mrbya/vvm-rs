---
id: TASK-54
title: >-
  Audit and harden runtime lifecycle, exception, thread, aliasing, and buffer
  invariants
status: To Do
assignee: []
created_date: '2026-08-11 11:57'
labels: []
milestone: m-6
dependencies: []
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
- [ ] #1 Context and model lifetime is explicitly validated
- [ ] #2 Finalization ordering and repeated finish behaviour are validated
- [ ] #3 Trace lifetime and failure behaviour are validated
- [ ] #4 Exception and panic boundaries are reviewed and hardened where needed
- [ ] #5 Thread behaviour and Send or Sync policy are explicit and enforced
- [ ] #6 Aliasing, buffer-size, and inout invariants are validated with regression coverage
<!-- AC:END -->

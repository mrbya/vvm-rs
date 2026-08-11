---
id: TASK-54
title: Audit generated-code safety and lint allowances
status: To Do
assignee: []
created_date: '2026-08-11 11:57'
labels: []
milestone: m-6
dependencies: []
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
- [ ] #1 Generated Rust safety is audited across representative fixture shapes
- [ ] #2 Generated C++ safety is audited across representative fixture shapes
- [ ] #3 Generated and handwritten lint allowances are inventoried
- [ ] #4 Obsolete allowances are removed
- [ ] #5 Necessary allowances remain narrowly scoped and justified
- [ ] #6 Generated snapshot and fixture outputs remain deterministic
<!-- AC:END -->

---
id: TASK-34
title: Benchmark vvm-build metadata and code generation
status: To Do
assignee: []
created_date: '2026-08-05 15:42'
labels:
  - benchmarking
  - criterion
  - vvm-build
milestone: m-4
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add deterministic Criterion benchmarks for vvm-build metadata decoding validation normalization type mapping identifier handling and generated-code construction. Use committed fixtures covering representative metadata shapes without widening public APIs solely for benchmarking.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Representative metadata shapes including signed wide packed aggregate array and inout forms are covered
- [ ] #2 Metadata JSON decoding and validation are benchmarked
- [ ] #3 Metadata normalization type mapping and identifier normalization are benchmarked
- [ ] #4 Generated C++ adapter CXX bridge and Rust wrapper construction are benchmarked where practical
- [ ] #5 Benchmarks use deterministic committed fixtures
- [ ] #6 Public APIs are not widened solely for benchmarking
<!-- AC:END -->

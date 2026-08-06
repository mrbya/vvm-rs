---
id: TASK-49
title: 'Add persisted-schema, generated-code, and CLI compatibility fixtures'
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 14:48'
labels:
  - release
  - compatibility
  - tests
milestone: m-5
dependencies:
  - TASK-48
priority: high
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Strengthen compatibility protection with deterministic fixtures and focused tests for persisted coverage artifacts, generated wrapper contracts, and machine-consumed CLI outputs. Scope includes representative versioned coverage schema fixtures, unknown-schema rejection tests, generated-code contract fixtures for supported DUT shapes, normalized nondeterministic data, and stable CLI contract checks such as metric lines, artifact layout, file names, exit behavior, and JSON outputs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Representative supported coverage schema fixtures exist and current readers and writers validate against them
- [ ] #2 Generated-code compatibility fixtures cover minimal scalar, wide, aggregate or complex port, inout, timing, and multi-clock shapes where the public contract differs
- [ ] #3 Nondeterministic paths or environment data are normalized so accidental compatibility drift fails clearly
- [ ] #4 CLI machine-consumed contract checks cover stable metric output, artifact layout, required file names, exit behavior, and supported JSON outputs
<!-- AC:END -->

---
id: TASK-43
title: Add release-shape external consumer fixtures for packaged workflows
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
labels:
  - release
  - fixtures
  - tests
milestone: m-5
dependencies: []
priority: high
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create or extend clean external fixtures that validate supported public workflows against packaged crate shapes instead of workspace path dependencies. Scope includes a generated counter consumer, a coverage-enabled consumer, a timing-enabled consumer, a multi-clock consumer, and isolated cargo-vvm installation and execution from packaged sources. Fixtures must explicitly opt out of the parent workspace, use only public APIs, isolate targets, and avoid globally installed cargo-vvm.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Packaged generated-counter, coverage, timing, and multi-clock consumers all build and run from packaged crate shapes
- [ ] #2 The packaged cargo-vvm crate can be installed into an isolated CARGO_HOME and its supported commands execute successfully
- [ ] #3 Fixtures opt out of the parent workspace, isolate target directories, avoid repository path dependencies, and use only public APIs
- [ ] #4 Fixture coverage is integrated into release-equivalent validation commands
<!-- AC:END -->

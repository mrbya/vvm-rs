---
id: TASK-46
title: Add release-shape external consumer fixtures for packaged workflows
status: In Progress
assignee:
  - OpenCode
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 15:16'
labels:
  - release
  - fixtures
  - tests
milestone: m-5
dependencies:
  - TASK-44
  - TASK-45
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

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing fixture workspaces under `tests/fixtures/` and identify which ones can be repurposed for packaged external-consumer validation versus which new minimal fixtures are needed.
2. Extend the packaged-consumer test harness to drive packaged `vvm-build` and `vvm-rs` through clean external workflows for generated counter, timing, and multi-clock scenarios without repository path dependencies.
3. Add a packaged coverage workflow fixture that exercises the public coverage API plus packaged `cargo-vvm` merge and reporting behavior from an isolated environment.
4. Add an isolated packaged `cargo-vvm` installation check using a temporary `CARGO_HOME`, ensuring commands run from the packaged crate shape rather than a globally installed binary.
5. Integrate the new packaged workflow fixtures into the release-facing validation surface and re-run the focused fixture suite until all packaged workflows pass.
<!-- SECTION:PLAN:END -->

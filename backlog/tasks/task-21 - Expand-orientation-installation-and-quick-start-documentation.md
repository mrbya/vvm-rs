---
id: TASK-21
title: Expand orientation installation and quick-start documentation
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 10:22'
labels: []
milestone: m-0
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Write a full introduction, installation path, and source-backed quick start that takes a new user from prerequisites through a complete first VVM test. The quick start must use a real tested workflow, explain tracing and replay, and point readers to the next guides without leaving key steps as pseudocode placeholders.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The introduction explains VVM architecture workflow audience maturity non-goals platform support and limitations honestly
- [x] #2 Installation guidance is accurate for users and contributors
- [x] #3 The quick start walks through a complete working VVM project and test without essential pseudocode placeholders
- [x] #4 Tracing replay and result interpretation are included in the first-user path
- [x] #5 Examples and code snippets are compiled or sourced from tested maintained example code
- [x] #6 Initial troubleshooting covers common setup and first-run failures
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Expand the introduction into an honest orientation chapter covering VVM's purpose, audience, maturity, architecture, non-goals, and current limitations.
2. Split setup guidance into installation and quick-start chapters so prerequisites, dependency configuration, and first-project steps are explicit.
3. Build the quick start around the maintained counter example, using real source-backed code for the DUT, `build.rs`, drive/sample/clock types, sequence, reference model, scoreboard, registered test, tracing, replay, and run commands.
4. Add first-user troubleshooting for missing Verilator, broken include-name alignment, unsupported HDL shapes, and tracing/replay setup.
5. Validate that every substantial snippet is sourced from maintained code or otherwise compiled.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded the orientation path into `introduction.md`, `installation.md`, `quick-start.md`, and a compatibility-preserving `getting-started.md` overview. The new quick start walks a new user through prerequisites, project layout, dependencies, `build.rs`, generated-DUT inclusion, typed drive/sample/clock setup, test registration, tracing, replay, and first-run troubleshooting. The orientation chapters now explain VVM's audience, maturity, non-goals, platform support, and current limitations honestly while pointing readers to the next sections.
<!-- SECTION:FINAL_SUMMARY:END -->

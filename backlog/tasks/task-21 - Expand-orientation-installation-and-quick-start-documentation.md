---
id: TASK-21
title: Expand orientation installation and quick-start documentation
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 17:33'
labels: []
milestone: m-1
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
1. Rewrite the orientation path around Introduction, Why VVM, HDL workflow fit, runtime model, Rust essentials, Installation, and an independent Quick Start.
2. Replace the old example-first quick start with a source-backed isolated fixture that creates a new verification crate from scratch and compiles exactly as shown.
3. Keep the first-user path deterministic and complete: HDL, build.rs, generated wrapper inclusion, typed drive/sample/clock, sequence, reference model, scoreboard, registered test, run command, trace command, and generated-artifact explanation.
4. Repurpose the long-lived getting-started entry page into a useful landing page with no internal milestone or migration language.
5. Validate the rewritten path with mdBook build and the dedicated documentation fixture test before finalizing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened because the current Quick Start still teaches through the curated counter example instead of an independent from-scratch project and the getting-started landing page still contains internal migration language.

Created the independent `tests/fixtures/docs-quick-start/` documentation fixture, rewrote `quick-start.md` around that fixture, and verified the exact project shown in the chapter by running the dedicated fixture test.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the user-centred orientation and quick-start rewrite. Added dedicated early-book chapters for product positioning, HDL toolchain fit, runtime mental model, and Rust essentials, then rebuilt the Quick Start around an independent `event_counter` verification crate rather than the curated counter example. Added the tested `tests/fixtures/docs-quick-start/` fixture with source anchors for `Cargo.toml`, `build.rs`, HDL, typed stimulus/observation/clock definitions, deterministic and replayable sequences, reference model, and registered tests. Repurposed `getting-started.md` into a useful landing page with no milestone or migration language. Validated the result with `mdbook build` and the dedicated fixture test.
<!-- SECTION:FINAL_SUMMARY:END -->

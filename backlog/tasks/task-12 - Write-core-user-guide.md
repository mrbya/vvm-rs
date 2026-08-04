---
id: TASK-12
title: Write core user guide
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-29 14:12'
updated_date: '2026-07-30 17:33'
labels: []
milestone: m-1
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Document normal VVM DUT integration, testbench authoring, test discovery, and supported generated-port behavior.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Core DUT and build workflow is documented
- [x] #2 Testbench models scoreboards clocks and tests are documented
- [x] #3 Source-backed examples and book validation are present
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite the core workflow chapters so each one teaches a generalized pattern directly instead of deferring to curated examples.
2. Reuse the independent docs quick-start fixture for source-backed snippets covering setup, build, inclusion, typed drive/sample, clocking, sequence, model, scoreboard, and registered test assembly.
3. Explain Cargo and Rust concepts only where they are needed for the verification workflow, with HDL-oriented language and cross-links to Rust essentials.
4. Add common mistakes, lifecycle notes, and diagnostics to each major chapter so readers can apply the workflow without source-code archaeology.
5. Validate the rewritten chapters through mdBook build and the documentation fixture test before finalizing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current core user guide summarizes the workflow but does not yet provide the full conceptual and workflow documentation promised by the task acceptance criteria.

Reopened because multiple core workflow chapters remain too terse to meet the user-centred teaching requirement. Project Setup, Build Script, Including the DUT, Driving Inputs, Sampling Outputs, Creating a Testbench, Registering Tests, and Configuring Tests all need substantial generalized instruction.

Replaced the summary-only core workflow chapters with source-backed chapters for project setup, build scripts, generated-wrapper inclusion, typed driving and sampling, testbench assembly, test registration, and run-time configuration. The new chapters reuse the independent docs quick-start fixture so the code shown in the book is compiled exactly as documented.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the core user-guide rewrite. Expanded `guide/project-setup.md`, `guide/build-script.md`, `guide/including-the-dut.md`, `guide/driving-inputs.md`, `guide/sampling-outputs.md`, `guide/creating-a-testbench.md`, `guide/registering-tests.md`, and `guide/configuring-tests.md` into generalized instructional chapters that explain the workflow directly for HDL engineers rather than sending readers into curated examples first. Added HDL-oriented explanations of Cargo and Rust concepts where needed, documented common mistakes and lifecycle behaviour, and backed the major code snippets with the tested docs quick-start fixture. Validated the rewritten core path with mdBook build and the dedicated fixture tests.
<!-- SECTION:FINAL_SUMMARY:END -->

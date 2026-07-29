---
id: TASK-12
title: Write core user guide
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-29 14:21'
labels: []
milestone: m-0
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
Document normal DUT integration, transaction traits, testbench ordering, and Cargo test discovery from current public APIs.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the core user-guide material for DutBuilder, generated inclusion, Drive, Sample, clocks, testbenches, models, scoreboards, and standard Rust test discovery. Book build passes.
<!-- SECTION:FINAL_SUMMARY:END -->

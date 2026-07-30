---
id: TASK-22
title: Write the public API usage guide for the VVM facade and direct-entry crates
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
Create a usage-oriented API guide that inventories the major public facade subsystems and explains how to use them correctly without forcing readers to infer workflows from rustdoc or examples alone. Cover generated DUT integration, build APIs, derives, test registration, testbench composition, randomization, tracing, timing, inout behavior, coverage, configuration, and errors, while linking to exact rustdoc types and methods.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every major facade subsystem and direct-entry package entry point is inventoried
- [x] #2 Major public APIs have usage-oriented book coverage with source-backed examples
- [x] #3 Book chapters link to exact rustdoc types and methods where appropriate
- [x] #4 Derives and attributes are documented with supported shapes compile-time expectations and common errors
- [x] #5 Configuration randomization tracing timing inout coverage and error APIs are covered
- [x] #6 Readers can understand the normal API workflow without reverse-engineering source or examples
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory the `vvm` facade modules and direct-entry crate surfaces into a dedicated API guide section organized by usage rather than by rustdoc module listing.
2. Add chapters for the facade/prelude, `DutBuilder`, generated DUT contract, drive/sample, clocks, testbench composition, models and scoreboards, the test attribute, configuration/context, randomization, tracing, schedulers, inout behavior, coverage, reports, and errors.
3. Link each chapter to the exact rustdoc module or type pages instead of copying signature lists into the book.
4. Use curated example code and public API tests as the canonical usage backing for each subsystem.
5. Document supported derive shapes, attribute forms, and common diagnostics where users need them to succeed.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added a dedicated `api-guide/` section covering the `vvm` facade modules and the major direct-entry workflows for `DutBuilder`, generated DUTs, drive/sample derives, clocks, testbench composition, models and scoreboards, the test attribute, runtime configuration, randomization, tracing, schedulers, inout handling, coverage, reports, and errors. The guide links readers to the exact rustdoc module surfaces while explaining how the APIs fit together in normal workflows so users do not have to reverse-engineer examples or source to get started.
<!-- SECTION:FINAL_SUMMARY:END -->

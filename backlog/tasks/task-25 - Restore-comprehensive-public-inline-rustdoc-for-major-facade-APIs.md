---
id: TASK-25
title: Restore comprehensive public inline rustdoc for major facade APIs
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-04 13:15'
updated_date: '2026-08-04 14:51'
labels: []
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit the public VVM facade and direct-entry package rustdoc, then restore self-contained documentation for the major public APIs that users encounter through IDE hovers, cargo doc, docs.rs, and API Guide links. The pass must cover the documented macro, derive, builder, testbench, configuration, scheduling, inout, coverage, and major error surfaces named in milestone 12.5.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Crucial public APIs have substantial self-contained rustdoc with focused examples
- [x] #2 #[vvm::test] documents supported attributes signatures return forms environment interaction and compile-time expectations
- [x] #3 Drive Sample Clock and Coverage derive docs describe helper attributes supported shapes generated implementations and common diagnostics
- [x] #4 include_dut! DutBuilder Testbench TestRunConfig TestContext scheduling APIs inout APIs coverage APIs and major public errors have materially expanded rustdoc
- [x] #5 Strict rustdoc builds and public doc examples pass without blanket lint suppression
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the public facade and direct-entry package sources for the high-priority items named in milestone 12.5, starting with `#[vvm::test]`, derive macros, `include_dut!`, `DutBuilder`, `Testbench`, configuration/context types, scheduling/timing types, inout APIs, coverage APIs, and major public error types.
2. Expand rustdoc in the smallest correct source files so the documentation is self-contained for IDE hovers, `cargo doc`, docs.rs, and API Guide backlinks.
3. Add focused examples that compile where feasible and use `no_run` only when native setup is required.
4. Keep exact supported macro and derive contracts in rustdoc rather than deferring to the book.
5. Rebuild strict public and internal rustdoc plus doctest-style validation after the expansion.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Restored substantial inline public rustdoc across the major facade APIs. Expanded `crates/vvm/src/lib.rs` to document the `Drive`, `Sample`, `Clock`, and `Coverage` derives, the `#[vvm::test]` attribute macro, and `include_dut!` with supported attributes, responsibilities, and focused examples. Added comprehensive struct-level and lifecycle docs to `crates/vvm-build/src/builder.rs` for `DutBuilder`, and expanded major runtime docs in `crates/vvm-core/src/registry.rs` (`TestRunConfig`), `test_context.rs` (`TestContext`), `clock/scheduler.rs` (`ClockScheduler`), `inout.rs` (`InoutState`), and `testbench.rs` (`Testbench`). These changes make IDE hover text and `cargo doc` materially more useful without forcing readers back into the book for the core contract. Validation: `just docs-api`, `just docs-internal`, `just doctest`, `just docs-test`, `just docs-links`, and the full repository gate including `just ci` passed. Retained limitation: examples for derives and `include_dut!` in `vvm` rustdoc are shown as non-doctested code blocks because they require generated DUT wrappers that are not available inside the facade crate’s doctest environment.
<!-- SECTION:FINAL_SUMMARY:END -->

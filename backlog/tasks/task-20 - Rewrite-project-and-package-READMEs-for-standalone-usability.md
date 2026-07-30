---
id: TASK-20
title: Rewrite project and package READMEs for standalone usability
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
Rewrite the root README and the vvm-rs, vvm-build, and cargo-vvm package READMEs so each one is useful when rendered in isolation. Keep the root README concise but self-sufficient, move detailed guidance into the book, and ensure commands, examples, and external links remain valid from GitHub, GitLab, and crates.io-style rendering.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The root README explains what VVM is why to use it requirements installation minimal workflow examples docs and limitations
- [x] #2 The vvm-rs README explains the facade crate workflow and links to the book examples and API reference
- [x] #3 The vvm-build README explains build.rs usage generated naming configuration output and common failures
- [x] #4 The cargo-vvm README explains installation workflow outputs merge policy failure behavior CI usage and troubleshooting
- [x] #5 Commands examples and links are valid in external README rendering contexts
- [x] #6 Detailed information removed from the root README exists in the book
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite the root README as a bounded project overview with requirements, installation, a minimal complete workflow, example ladder, documentation links, development commands, and honest limitations.
2. Rewrite `crates/vvm/README.md` as facade-crate documentation with dependency placement, generated-DUT relationship, module overview, and a compact end-to-end workflow.
3. Rewrite `crates/vvm-build/README.md` around the normal `build.rs` workflow, name matching, source configuration, generated output, timing/tracing options, and common failures.
4. Rewrite `crates/cargo-vvm/README.md` around the common coverage workflow, command syntax, outputs, merge policy, failure semantics, CI usage, and troubleshooting.
5. Reconcile every detailed topic removed from the root README with the new book chapters and validate external-rendering links.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Rewrote the root `README.md` as a bounded project overview with requirements, installation, a minimal workflow, example ladder, docs links, development commands, and explicit alpha limitations. Rewrote `crates/vvm/README.md`, `crates/vvm-build/README.md`, and `crates/cargo-vvm/README.md` so each package now explains its role, common workflow, key commands or configuration, and where to find the full book and API docs. The README surfaces now align with the new book structure instead of carrying the only detailed explanations for major features.
<!-- SECTION:FINAL_SUMMARY:END -->

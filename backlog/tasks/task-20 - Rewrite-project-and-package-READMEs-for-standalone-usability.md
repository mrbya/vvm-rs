---
id: TASK-20
title: Rewrite project and package READMEs for standalone usability
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 17:33'
labels: []
milestone: Milestone 12.5 — User-centred documentation rewrite
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
1. Reconcile the root README and package READMEs with the final user-centred book narrative so they explain what VVM is, where it fits, and where to read more without carrying placeholder or stale example-first guidance.
2. Remove incomplete code sketches and internal wording from README surfaces, replacing them with valid minimal patterns or links to the authoritative book chapters.
3. Keep each package README independently useful by preserving crate-specific workflow, support notes, and links to the relevant guide chapters and rustdoc.
4. Recheck every command and external link after the book rewrite so README surfaces remain valid in isolation.
5. Finalize the task after the README surfaces and public book communicate the same positioning and workflow story.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened because the root and package README surfaces must be realigned to the final user-centred positioning after the public book rewrite and validation pass.

Removed the placeholder README quick-start code path from the root README, aligned the overview messaging with the new Why/Fit/How chapters, and kept the package READMEs crate-specific while linking to the authoritative book chapters.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the final README alignment pass. Updated the root README so it points readers to the authoritative from-scratch Quick Start and Why VVM positioning chapters instead of carrying a partial example-first quickstart with placeholders. Kept the root README concise but self-sufficient on purpose, requirements, crate roles, examples, docs links, and alpha limitations. Preserved crate-specific README guidance for `vvm-rs`, `vvm-build`, and `cargo-vvm` while ensuring the book now owns the detailed workflow explanations. Revalidated the README-linked documentation surfaces through the docs-site and docs-links checks.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-6
title: Document cargo-vvm coverage command
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
Create book-only cargo-vvm command documentation covering syntax outputs policies and failure semantics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Installation command reference and output layout are documented
- [x] #2 Merge policy and exit behavior are explicit
- [x] #3 GitLab coverage integration is documented
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Expand the cargo-vvm book documentation into installation, workflow, command reference, output layout, merge policy, failure semantics, GitLab integration, and troubleshooting chapters.
2. Rewrite the content around the actual CLI behavior from `crates/cargo-vvm/src/cli.rs`, orchestration code, and the maintained counter/FIFO workflows.
3. Explain output-directory isolation, per-test artifacts, merged JSON, text/HTML reports, metric output, provenance, fingerprints, retry restrictions, child-status preservation, reporting failures, and no-artifact runs.
4. Provide a concise command synopsis and a small GitLab CI example that match the current CLI.
5. Validate the command descriptions against the code and the maintained coverage workflows.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened during milestone 12.5 completion audit. The current cargo-vvm documentation is not yet a full workflow and command reference at the depth implied by the task acceptance criteria.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Expanded `cargo-vvm` documentation into a real command guide with installation, workflow, command reference, output layout, merge policies, failure semantics, GitLab CI usage, and troubleshooting chapters. The book now documents the actual CLI behavior from the current binary surface instead of only a short summary page, and the package README mirrors that workflow for standalone use.
<!-- SECTION:FINAL_SUMMARY:END -->

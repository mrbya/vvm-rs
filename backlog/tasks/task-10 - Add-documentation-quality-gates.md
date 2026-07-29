---
id: TASK-10
title: Add documentation quality gates
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-29 14:23'
labels: []
milestone: m-0
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add automated documentation compilation and generated-site link checks with bounded external-link policy.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Doctests book validation and link checks run locally
- [ ] #2 Generated site and cross-link checks are enforced
- [ ] #3 External-link policy is bounded and documented
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add local documentation build, doctest, strict rustdoc, generated-site entry-point checks, and CI execution.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added docs-test, docs-links, strict public rustdoc, and generated-site assertions. Validated `just docs-test`, `just docs-links`, `just fmt --check`, and `just check -- -D warnings`.
<!-- SECTION:FINAL_SUMMARY:END -->

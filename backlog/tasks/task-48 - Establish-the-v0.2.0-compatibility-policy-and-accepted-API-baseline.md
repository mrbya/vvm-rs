---
id: TASK-48
title: Establish the v0.2.0 compatibility policy and accepted API baseline
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 14:48'
labels:
  - release
  - compatibility
  - api
milestone: m-5
dependencies:
  - TASK-47
priority: high
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define the canonical public compatibility policy for the v0.2.0 cycle and accept the reviewed post-12.6 API surface as the baseline for future checks. Scope includes Rust API policy, patch compatibility within 0.2.x, persisted coverage schema policy, generated-code compatibility policy, CLI machine-consumed compatibility policy, public API inventory review, accidental export review, and contributor-facing API-diff tooling and commands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A canonical compatibility policy documents Rust API, pre-1.0 semver, 0.2.x patch compatibility, coverage schema, generated-code, and CLI compatibility rules
- [ ] #2 The reviewed current public API is accepted as the v0.2.0 baseline and any accidental exports are either removed with justification or explicitly accepted
- [ ] #3 Contributor commands and tooling can compare a branch or later release candidate against an explicit v0.2.0 baseline without depending on a nonexistent v0.2.0 tag
- [ ] #4 User-facing book content and package docs do not contradict the compatibility policy
<!-- AC:END -->

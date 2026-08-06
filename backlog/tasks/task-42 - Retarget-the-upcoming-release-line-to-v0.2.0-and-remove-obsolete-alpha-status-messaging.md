---
id: TASK-42
title: >-
  Retarget the upcoming release line to v0.2.0 and remove obsolete alpha status
  messaging
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
labels:
  - release
  - docs
  - status
milestone: m-5
dependencies: []
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Update the repository’s forward-looking release references from the old v0.1.0 line to the upcoming v0.2.0 line, adopt a coherent development-version strategy, and remove obsolete project-wide alpha-stage messaging while preserving historically accurate references to v0.1.0-alpha.1. Scope includes the workspace version strategy, roadmap text, README and package README status language, mdBook introduction and compatibility pages, version snippets, changelog framing, and a regression check that prevents obsolete project-wide alpha wording from returning in current public documents.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Forward-looking release references consistently target v0.2.0 and v0.2.0-rc.1 where applicable
- [ ] #2 The workspace uses one coherent development version strategy for the v0.2.0 cycle and internal dependency requirements remain aligned
- [ ] #3 Historical references to v0.1.0-alpha.1 remain accurate and are not blindly rewritten
- [ ] #4 Current public README and book status wording no longer presents VVM as an alpha prototype and instead describes it as pre-1.0 with an established v0.2.0 API freeze
- [ ] #5 A repository check rejects obsolete project-wide alpha wording in current public documentation while allowing explicit historical references
- [ ] #6 CHANGELOG.md is updated to keep the historical alpha release entry intact and prepare Unreleased for the v0.2.0 line
<!-- AC:END -->

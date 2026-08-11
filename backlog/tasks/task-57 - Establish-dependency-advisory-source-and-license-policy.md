---
id: TASK-57
title: 'Establish dependency, advisory, source, and license policy'
status: To Do
assignee: []
created_date: '2026-08-11 11:57'
labels: []
milestone: m-6
dependencies: []
documentation:
  - Cargo.toml
  - Cargo.lock
  - justfile
  - CONTRIBUTING.md
  - crates/vvm-macros/Cargo.toml
  - crates/cargo-vvm/Cargo.toml
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Perform the final dependency audit for the publishable v0.2.0 line. Preserve existing working commands such as just audit and just unused, add cargo-deny with a repository deny.toml and contributor-facing recipe, review advisories, licenses, sources, duplicate dependencies, feature defaults, and published dependency footprints, and document any justified exceptions explicitly.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 cargo audit passes or has explicitly justified exceptions
- [ ] #2 cargo deny check passes
- [ ] #3 cargo udeps passes
- [ ] #4 deny.toml is committed
- [ ] #5 License policy matches the resolved dependency graph
- [ ] #6 Source policy is enforced and unapproved Git dependencies are absent
- [ ] #7 Duplicate dependencies are reviewed
- [ ] #8 Default features and publishable dependency footprints are reviewed
<!-- AC:END -->

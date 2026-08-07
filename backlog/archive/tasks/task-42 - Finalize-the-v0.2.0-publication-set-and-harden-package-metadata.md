---
id: TASK-42
title: Finalize the v0.2.0 publication set and harden package metadata
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
labels:
  - release
  - packaging
milestone: m-5
dependencies: []
priority: high
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Confirm the intended published crate set, derive the real publication order from the dependency graph, make cargo-vvm publishable, and audit metadata for every published crate. Scope includes per-crate descriptions, readmes, docs/homepage/repository/license metadata, keywords and categories, include/exclude policy, archive-facing support files, and keeping examples and fixtures nonpublishable.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The publication set and publication order are explicit and derived from actual package dependencies
- [ ] #2 cargo-vvm is publishable under its intended package name and example or fixture packages remain excluded from publication
- [ ] #3 Every published package has package-specific descriptions and valid readme, documentation, homepage, repository, license, keyword, category, and rust-version metadata
- [ ] #4 Published package archives are configured to include required license and native support files while excluding development-only or irrelevant files
- [ ] #5 Package metadata and docs consistently target the v0.2.0 release line
<!-- AC:END -->

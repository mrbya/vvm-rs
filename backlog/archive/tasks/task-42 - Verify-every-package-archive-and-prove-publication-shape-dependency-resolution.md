---
id: TASK-42
title: Verify every package archive and prove publication-shape dependency resolution
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
labels:
  - release
  - packaging
  - tests
milestone: m-5
dependencies: []
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Exercise cargo package, cargo package --list, and cargo publish --dry-run for every intended published package, then extend the package-consumer infrastructure into a release-shape simulation that resolves only from packaged crate contents in the real publication order. Scope includes manual file-list review, staged dependency resolution without workspace fallbacks, isolated targets, and proving that packaged crates do not require workspace inheritance after packaging.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each intended published crate passes cargo package --list, cargo package, and cargo publish --dry-run
- [ ] #2 Package file lists are reviewed and required support files are present without leaking workspace-only paths or irrelevant files
- [ ] #3 Publication-shape simulation resolves vvm-core, vvm-macros, vvm-build, vvm-rs, and cargo-vvm from packaged crate contents in dependency order
- [ ] #4 Isolated temporary directories and isolated CARGO_TARGET_DIR values are used for package-consumer validation
- [ ] #5 Failure diagnostics make missing packaged resources or dependency mismatches obvious
<!-- AC:END -->

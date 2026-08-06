---
id: TASK-47
title: Define and document the supported compatibility matrix for v0.2.0
status: To Do
assignee: []
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 14:48'
labels:
  - release
  - compatibility
  - docs
milestone: m-5
dependencies:
  - TASK-51
priority: high
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Establish one explicit support matrix covering Rust, Verilator, C++ compilers, operating systems, and timing-mode native requirements using only configurations that are actually validated by the repository. Scope includes updating user-facing docs, aligning manifest metadata with the tested matrix, and ensuring the documented support surface matches the current CI and local validation strategy.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The MSRV and current stable Rust support are explicit and aligned with rust-version metadata
- [ ] #2 Minimum and current supported Verilator versions are documented and validated
- [ ] #3 Supported Linux and tested GCC and Clang toolchains are documented without advertising untested operating systems or compiler families
- [ ] #4 Timing-enabled model native requirements are explicit and consistent across docs and CI
- [ ] #5 User-facing compatibility documentation matches the actual validated matrix
<!-- AC:END -->

---
id: TASK-47
title: Define and document the supported compatibility matrix for v0.2.0
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 16:35'
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
- [x] #1 The MSRV and current stable Rust support are explicit and aligned with rust-version metadata
- [x] #2 Minimum and current supported Verilator versions are documented and validated
- [x] #3 Supported Linux and tested GCC and Clang toolchains are documented without advertising untested operating systems or compiler families
- [x] #4 Timing-enabled model native requirements are explicit and consistent across docs and CI
- [x] #5 User-facing compatibility documentation matches the actual validated matrix
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the current user-facing compatibility claims across the book, top-level README, crate READMEs, manifests, and CI so the documented support surface matches what the repository actually validates.
2. Tighten the authoritative compatibility chapter into one explicit tested matrix covering Rust, Verilator, Linux, GCC, Clang status, and timing-mode native requirements, using only support claims backed by CI or local validation policy.
3. Align adjacent user-facing docs and package-facing text with that matrix so no public page implies unsupported operating systems or compiler families.
4. Re-run focused documentation and search checks to confirm the compatibility wording is consistent before closing the task.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reworked the public compatibility chapter into an explicit tested matrix covering Linux-only support, Rust 1.87.0 plus current stable Rust, Verilator 5.000 minimum and 5.050 current validation, GCC in native CI, Clang Linux native-fixture validation, and the C++17 versus timing-mode C++20 requirements.

Aligned the top-level README, `crates/vvm/README.md`, `docs/book/src/installation.md`, `docs/book/src/introduction.md`, and `docs/book/src/guide/timing-models.md` with the same bounded support claims so user-facing docs no longer imply broader platform support than the repository validates.

Validation: `mdbook build docs/book`, `clang++ --version`, `verilator --version`, and `CXX=clang++ cargo test --all-features -p vvm-rs --test fixtures native_`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Published one explicit v0.2.0 compatibility matrix centered on the authoritative compatibility chapter, then aligned the README, facade crate README, installation page, introduction, and timing guidance to the same support surface. The documented matrix now matches repository validation: Linux-only native support, Rust 1.87.0 plus current stable Rust, Verilator 5.000 minimum with 5.050 current validation, GCC in native CI, Clang validated through Linux native fixtures, and timing mode requiring a C++20 coroutine-capable compiler.
<!-- SECTION:FINAL_SUMMARY:END -->

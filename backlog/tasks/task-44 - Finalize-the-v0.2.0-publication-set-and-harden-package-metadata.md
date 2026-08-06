---
id: TASK-44
title: Finalize the v0.2.0 publication set and harden package metadata
status: Done
assignee:
  - OpenCode
created_date: '2026-08-06 14:47'
updated_date: '2026-08-06 15:01'
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
- [x] #1 The publication set and publication order are explicit and derived from actual package dependencies
- [x] #2 cargo-vvm is publishable under its intended package name and example or fixture packages remain excluded from publication
- [x] #3 Every published package has package-specific descriptions and valid readme, documentation, homepage, repository, license, keyword, category, and rust-version metadata
- [x] #4 Published package archives are configured to include required license and native support files while excluding development-only or irrelevant files
- [x] #5 Package metadata and docs consistently target the v0.2.0 release line
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit each intended published crate manifest, local README situation, and crate role so the publication set and publish order reflect the actual dependency graph rather than assumptions.
2. Update package metadata across `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`, and `cargo-vvm`, including package-specific descriptions, readmes, documentation/homepage/repository links, keywords or categories where missing, and publish settings.
3. Add or refine package-local READMEs for crates that do not yet have a standalone crates.io-facing document, and ensure examples or fixtures remain nonpublishable.
4. Configure package include or exclude policy where needed so required licenses and support files are retained while development-only content stays out of publishable archives.
5. Document the publication set and derived publication order in repository documentation, then run focused manifest and package-metadata validation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Derived the publication order from the real dependency graph reported by `cargo metadata`: `vvm-core -> vvm-macros -> vvm-build -> vvm-rs -> cargo-vvm`.

Confirmed the existing published alpha crates on crates.io for `vvm-core`, `vvm-macros`, `vvm-build`, and `vvm-rs`, and confirmed that `cargo-vvm` currently has no crates.io record.

Added standalone crate-local READMEs and documentation URLs for `vvm-core` and `vvm-macros`, removed the `publish = false` block from `cargo-vvm`, and embedded dual-license texts into every publishable crate root so package archives carry license files.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Finalized the intended publishable crate set and hardened the publish-facing metadata for the `v0.2.0` line.

What changed:
- Confirmed the publishable set as `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`, and `cargo-vvm`.
- Derived the real publication order from the current dependency graph: `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`, then `cargo-vvm`.
- Removed `publish = false` from `crates/cargo-vvm/Cargo.toml`, added cargo-plugin-specific categories and keywords, and kept all example crates nonpublishable.
- Added missing standalone crate metadata for `vvm-core` and `vvm-macros`, including package-local READMEs and public documentation URLs.
- Tightened package-specific descriptions for `vvm-build` and `vvm-rs`.
- Added crate-root `LICENSE-MIT` and `LICENSE-APACHE` files to every publishable crate so package archives carry explicit license texts.
- Documented the primary entry crates, supporting crates, and publication order in `docs/book/src/development/contributing.md`.

Files affected:
- `crates/vvm-core/Cargo.toml`
- `crates/vvm-core/README.md`
- `crates/vvm-core/LICENSE-MIT`
- `crates/vvm-core/LICENSE-APACHE`
- `crates/vvm-macros/Cargo.toml`
- `crates/vvm-macros/README.md`
- `crates/vvm-macros/LICENSE-MIT`
- `crates/vvm-macros/LICENSE-APACHE`
- `crates/vvm-build/Cargo.toml`
- `crates/vvm-build/LICENSE-MIT`
- `crates/vvm-build/LICENSE-APACHE`
- `crates/vvm/Cargo.toml`
- `crates/vvm/LICENSE-MIT`
- `crates/vvm/LICENSE-APACHE`
- `crates/cargo-vvm/Cargo.toml`
- `crates/cargo-vvm/LICENSE-MIT`
- `crates/cargo-vvm/LICENSE-APACHE`
- `docs/book/src/development/contributing.md`

Package and publication policy established:
- Primary entry crates: `vvm-rs`, `vvm-build`, and `cargo-vvm`.
- Supporting published crates: `vvm-core` and `vvm-macros`.
- Example crates remain `publish = false` and are not part of the release set.
- Existing crates.io alpha ownership was confirmed for the four already-published crate names, and `cargo-vvm` currently has no crates.io record.

Commands executed:
- `cargo metadata --format-version 1 --no-deps`
- `cargo check --workspace`
- `cargo package --list -p vvm-core --allow-dirty`
- `cargo package --list -p vvm-macros --allow-dirty`
- `cargo package --list -p vvm-build --allow-dirty`
- `cargo package --list -p vvm-rs --allow-dirty`
- `cargo package --list -p cargo-vvm --allow-dirty`
- crates.io API checks for all intended published crate names

Validation results:
- Manifest metadata resolves cleanly through `cargo metadata`.
- The workspace still builds after the metadata updates.
- Package-list inspection shows the new README and license files are included in publishable crate archives.

Release implications:
- The repository now has one explicit publishable package set and order for the `v0.2.0` line.
- `cargo-vvm` is no longer blocked at the manifest level from publication.
- Later package dry-run and release-simulation tasks can validate archive behavior against the intended package surface.

Retained limitations:
- This task established metadata and publish policy; it did not yet complete full `cargo package` / `cargo publish --dry-run` validation or publication-shape consumer simulation, which remain in later milestone tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

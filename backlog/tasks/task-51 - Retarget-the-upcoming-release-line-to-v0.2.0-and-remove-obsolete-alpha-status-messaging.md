---
id: TASK-51
title: >-
  Retarget the upcoming release line to v0.2.0 and remove obsolete alpha status
  messaging
status: Done
assignee:
  - OpenCode
created_date: '2026-08-06 14:48'
updated_date: '2026-08-06 15:15'
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
- [x] #1 Forward-looking release references consistently target v0.2.0 and v0.2.0-rc.1 where applicable
- [x] #2 The workspace uses one coherent development version strategy for the v0.2.0 cycle and internal dependency requirements remain aligned
- [x] #3 Historical references to v0.1.0-alpha.1 remain accurate and are not blindly rewritten
- [x] #4 Current public README and book status wording no longer presents VVM as an alpha prototype and instead describes it as pre-1.0 with an established v0.2.0 API freeze
- [x] #5 A repository check rejects obsolete project-wide alpha wording in current public documentation while allowing explicit historical references
- [x] #6 CHANGELOG.md is updated to keep the historical alpha release entry intact and prepare Unreleased for the v0.2.0 line
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit forward-looking release-version references, current alpha-status wording, and package-version mismatches across manifests, docs, scripts, and CI-related files.
2. Update the workspace development version strategy to the v0.2.0 line, including cargo-vvm and any internal dependency references that still use the old line.
3. Replace obsolete project-wide alpha wording in current public docs with accurate pre-1.0 status language while preserving explicit historical references to v0.1.0-alpha.1.
4. Add a lightweight repository hygiene check for forbidden current-status alpha wording and wire it into the existing documentation or validation command surface.
5. Update CHANGELOG framing and any roadmap or release references touched by this task, then run focused validation for the new wording and version checks.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audited forward-looking release references and alpha-status wording across manifests, READMEs, book pages, roadmap text, and changelog framing before editing.

Moved the workspace and all internal crate dependencies to the `0.2.0-dev.0` development line and switched `cargo-vvm` to the shared workspace version.

Added a public-document wording guard to `scripts/check-doc-hygiene.sh` and verified it through the normal `just docs-links` path.

Reopened after package-simulation work revealed that Cargo cannot package unpublished interdependent prerelease versions for this workspace. `cargo package` rewrites path dependencies to registry dependencies and then fails to resolve internal `0.2.0-dev.0` versions that do not yet exist on crates.io. The workspace manifest version strategy needs to move to `0.2.0` while keeping publication itself gated to later release tasks.

Adjusted the manifest version strategy from `0.2.0-dev.0` to `0.2.0` after proving that Cargo cannot package or publish-dry-run unpublished interdependent prerelease crate versions for this workspace. The repository still treats `v0.2.0` as unreleased; the manifest version is final-version-shaped solely so package verification can succeed before publication.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Retargeted the repository’s development line and public status messaging to the `v0.2.0` cycle.

What changed:
- Updated the workspace package version and internal dependency requirements to the `0.2.0` release line and switched `crates/cargo-vvm/Cargo.toml` from its stale standalone `0.1.0` version to the shared workspace version.
- Rewrote current public status wording in `README.md`, `crates/vvm/README.md`, `docs/book/src/introduction.md`, `docs/book/src/why-vvm.md`, `docs/book/src/installation.md`, and `docs/book/src/guide/compatibility-and-limitations.md` so VVM is described as pre-`1.0` with a frozen `v0.2.0` release-cycle API instead of an alpha prototype.
- Updated forward-looking roadmap and release-line references in `docs/dev/implementation-plan.md`, `docs/dev/public-api.md`, and `docs/dev/example-strategy.md` from the old `v0.1.0` line to `v0.2.0` and `v0.2.0-rc.1` where appropriate.
- Prepared `CHANGELOG.md` for the new release line while preserving the historical `0.1.0-alpha.1` entry and comparison links.
- Extended `scripts/check-doc-hygiene.sh` with a regression guard that rejects obsolete project-wide alpha wording in current public docs.

Version-strategy decision:
- The repository initially moved to `0.2.0-dev.0`, but package verification work proved that Cargo cannot package unpublished interdependent prerelease crate versions for this workspace because path dependencies are rewritten to registry dependencies during packaging.
- To keep `cargo package` and `cargo publish --dry-run` viable before production publication, the workspace manifests now use `0.2.0` while release automation and contributor docs keep actual publication, tags, and release creation gated to later milestone tasks.

Files affected:
- `Cargo.toml`
- `Cargo.lock`
- `crates/cargo-vvm/Cargo.toml`
- `README.md`
- `crates/vvm/README.md`
- `crates/vvm-build/README.md`
- `docs/book/src/introduction.md`
- `docs/book/src/why-vvm.md`
- `docs/book/src/installation.md`
- `docs/book/src/guide/compatibility-and-limitations.md`
- `docs/dev/implementation-plan.md`
- `docs/dev/public-api.md`
- `docs/dev/example-strategy.md`
- `CHANGELOG.md`
- `scripts/check-doc-hygiene.sh`

Commands executed:
- `cargo check --workspace`
- `bash scripts/check-doc-hygiene.sh`
- `just docs-links`
- repository-wide grep audits for old release-line and alpha wording references
- package-verification experiments that established the final manifest-version strategy

Validation results:
- Workspace builds cleanly on the `0.2.0` release-line manifests.
- The documentation hygiene script passes with the new public-status wording guard enabled.
- Remaining `v0.1.0-alpha.1` references are intentional historical records in the changelog and task metadata.

Release implications:
- The repository now presents the upcoming release as `v0.2.0`, the release-candidate target as `v0.2.0-rc.1`, and the current public API as frozen for this release cycle.
- Manifest versions now match the final release line so package validation can run before publication, but no publication, tag creation, or release creation is implied or permitted by this task.

Retained limitations:
- This task did not complete the substantive changelog population, compatibility policy, package archive validation, or release tooling; those remain in their dedicated milestone tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

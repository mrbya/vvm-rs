---
id: TASK-57
title: Perform repository hygiene and stale-state cleanup
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:57'
updated_date: '2026-08-11 12:30'
labels: []
milestone: m-6
dependencies:
  - TASK-53
documentation:
  - README.md
  - CHANGELOG.md
  - CONTRIBUTING.md
  - docs/book/src/development/contributing.md
  - docs/book/src/guide/compatibility-and-limitations.md
  - examples/
  - tests/fixtures/
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit release-facing hygiene across docs, code, examples, fixtures, features, ignored tests, TODO-style markers, compatibility aliases, generated snapshots, package sizes, and public links. Resolve or explicitly classify every release-relevant stale-state finding without turning the milestone into a broad refactor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Release-relevant TODO and FIXME style notes are resolved or explicitly deferred
- [x] #2 Ignored tests are reviewed
- [x] #3 Dead feature flags and temporary compatibility aliases are removed or justified
- [x] #4 Examples, fixtures, generated artifacts, and lint allowances are reviewed
- [x] #5 Package sizes are recorded
- [x] #6 Public links and badges are valid
- [x] #7 No stale release-facing documentation remains
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Search the repository for release-relevant TODO/FIXME/HACK markers, ignored tests, feature flags, temporary aliases, package metadata status notes, and stale release wording.
2. Review the meaningful matches in public docs, examples, fixtures, and test harnesses; fix any clear release-facing drift and classify the rest.
3. Record package-size and link/badge status from the current repository state without inventing arbitrary new thresholds.
4. Run focused validation for the touched docs or fixture metadata, then finalize the task with the classified hygiene inventory.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Repository sweep found no release-relevant `TODO`, `FIXME`, `HACK`, or `XXX` markers in current crates, examples, fixtures, README, CONTRIBUTING, CHANGELOG, or book sources. Remaining matches are limited to the implementation plan and vendored reference-project material under `docs/dev/reference-projects/`, which are historical or vendor-only rather than release-facing work items.

There are no actual `#[ignore]` tests in the workspace; the only `ignore` mention in Rust sources is documentation describing ordinary Rust ignore semantics.

No crate feature tables are currently defined in workspace packages, so there are no dead feature flags to remove in this release line.

`publish = false` appears only on examples and test fixtures, which is intentional and consistent with their non-publishable role.

The inout `set_<port>` compatibility alias remains an intentional part of the supported API rather than a stale temporary shim, so it was kept and classified as valid.

Fixed the stale typo and drift in `docs/book/src/development/contributing.md` so the book now matches the current contributor bootstrap and command surface, including `cargo-deny`, `just deny`, and `just repro-check`.

Recorded current compressed package sizes from local `cargo package --locked --allow-dirty --no-verify` runs using the same patching approach needed for unpublished internal dependencies.

Validated public book and API links with `just docs-links`; there are no badges to review in the current top-level README surface.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the repository hygiene and stale-state audit for milestone 12.8.

What was reviewed:
- release-facing TODO-style markers across current crates, examples, fixtures, README, CONTRIBUTING, CHANGELOG, and book sources;
- ignored tests;
- feature-flag and compatibility-alias state;
- example and fixture publication metadata;
- generated snapshot and allowance status as covered by the earlier FFI/generated-code tasks;
- package archive sizes;
- public documentation and Pages link integrity.

Fixes:
- Updated `docs/book/src/development/contributing.md` to fix the stale typo in first-time setup and to align the contributor command/tool list with the current repository surface.

Classified findings:
- No release-relevant `TODO`, `FIXME`, `HACK`, or `XXX` markers remain in the current release-facing repository content.
- Remaining TODO-style matches are confined to the implementation plan and vendored reference-project material under `docs/dev/reference-projects/`; those are historical or vendor-only and are not release blockers.
- There are no actual `#[ignore]` tests in the workspace.
- There are no crate feature tables in the workspace packages, so there are no dead feature flags to prune for `v0.2.0`.
- `publish = false` is present only on examples and test fixtures, which is correct for their role.
- The generated inout `set_<port>` compatibility alias is an intentional supported convenience, not a stale temporary alias.

Recorded package sizes:
- `vvm-core-0.2.0.crate`: 139602 bytes
- `vvm-macros-0.2.0.crate`: 29092 bytes
- `vvm-build-0.2.0.crate`: 131700 bytes
- `vvm-rs-0.2.0.crate`: 29375 bytes
- `cargo-vvm-0.2.0.crate`: 28230 bytes

Commands and validation:
- repository grep sweeps for TODO-style markers, `#[ignore]`, `publish = false`, feature tables, and link domains;
- local `cargo package --locked --allow-dirty --no-verify` size collection for publishable crates, using patch config where unpublished internal dependencies require it;
- `just docs-links` to validate assembled public documentation and local Pages targets.

Accepted limitations:
- Vendor TODOs inside `docs/dev/reference-projects/` remain by design because that material is checked-in reference content, not maintained release-facing VVM implementation code.
- The implementation plan still contains open milestone checklist items; that is expected roadmap state rather than stale end-user documentation.
<!-- SECTION:FINAL_SUMMARY:END -->

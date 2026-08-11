---
id: TASK-58
title: 'Establish dependency, advisory, source, and license policy'
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:58'
updated_date: '2026-08-11 12:19'
labels: []
milestone: m-6
dependencies:
  - TASK-53
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
- [x] #1 cargo audit passes or has explicitly justified exceptions
- [x] #2 cargo deny check passes
- [x] #3 cargo udeps passes
- [x] #4 deny.toml is committed
- [x] #5 License policy matches the resolved dependency graph
- [x] #6 Source policy is enforced and unapproved Git dependencies are absent
- [x] #7 Duplicate dependencies are reviewed
- [x] #8 Default features and publishable dependency footprints are reviewed
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inspect the current dependency graph, duplicate versions, default feature usage, and publishable source policy from the workspace manifests and lockfile.
2. Run the existing dependency checks (`just audit`, `just unused`, and duplicate-tree review) to identify the current gaps before adding new policy.
3. Add a repository `deny.toml`, a `just deny` recipe, and contributor/bootstrap updates for `cargo-deny`, making the allow-list and source policy match the actual resolved dependency graph.
4. Re-run audit, deny, and udeps validation; classify duplicates and any accepted limitations; then close the task with the reviewed policy summary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Dependency audit evidence: `cargo audit` returned clean, `cargo +nightly udeps --all-targets --workspace` reported no unused dependencies, and `cargo deny check` passed with one reviewed duplicate warning for `syn`.

Added `deny.toml` with explicit advisory, license, bans, and source policy for the current graph; then tightened the license allow-list after the first `cargo deny` pass reported unmatched allowances.

Confirmed there are no Git sources in `Cargo.lock` and no unapproved Git dependencies in publishable crate manifests; the only patch usage is the intentional test-only packaged-consumer fixture override.

Reviewed duplicate dependencies from `cargo tree -d`: `syn` 2.x and 3.x are required by incompatible proc-macro stacks, while `rand` and `getrandom` duplicates are primarily runtime-vs-dev/test splits driven by `vvm-core` RNG choices and `proptest`/`tempfile`.

Reviewed publishable default features and footprint: none of the publishable crates define crate features today, `vvm-core` intentionally disables default features on `rand_core` and `rand_chacha`, `vvm-macros` keeps a minimal direct footprint of `proc-macro2`, `quote`, and `syn`, and `cargo-vvm` keeps its direct runtime footprint to `clap`, `serde`, `serde_json`, `thiserror`, and `vvm-core`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the dependency, advisory, source, and license audit for the milestone 12.8 pre-RC state.

What changed:
- Added `deny.toml` with explicit policy for advisories, licenses, duplicate-version reporting, and allowed sources.
- Added `just deny` to the maintainer command surface.
- Updated `just init` and `CONTRIBUTING.md` so contributor setup and release-facing validation now include `cargo-deny`.

Results:
- `cargo audit` passed with no accepted advisory exceptions required.
- `cargo deny check` passed. The only emitted issue is a reviewed duplicate-version warning for `syn`, which is intentionally tolerated under the bans policy rather than treated as a release blocker.
- `cargo +nightly udeps --all-targets --workspace` reported that all dependencies are used.
- No Git sources were present in `Cargo.lock`, and no publishable crate manifest depends on an unapproved Git source.

License and source policy:
- The resolved graph fits the committed allow-list in `deny.toml`: `Apache-2.0`, `MIT`, `Unicode-3.0`, and `Zlib`.
- The source policy now denies unknown registries and all Git sources while allowing only crates.io.

Duplicate dependency review:
- `syn` 2.x and 3.x are both present because the repository mixes a direct `syn` 2 proc-macro stack (`vvm-macros`) with upstream proc-macro dependencies already on `syn` 3 (`clap_derive`, `cxxbridge-macro`, `serde_derive`, `thiserror-impl`). This is not easily collapsible in-repo and is not a release blocker.
- `rand` / `rand_core` / `rand_chacha` and `getrandom` duplicates are mainly runtime-versus-dev/test splits between the repository’s own `rand_* 0.10` usage and upstream dev tooling such as `proptest` and `tempfile`.

Default features and footprint review:
- None of the publishable crates define crate feature tables today, so there is no hidden heavy default feature surface to trim.
- `vvm-core` intentionally disables default features on `rand_core` and `rand_chacha`.
- `vvm-macros` keeps the expected minimal direct proc-macro dependency set: `proc-macro2`, `quote`, and `syn` with `full`.
- `cargo-vvm` keeps a small direct runtime dependency footprint: `clap`, `serde`, `serde_json`, `thiserror`, and `vvm-core`.

Commands executed:
- `just audit`
- `cargo tree -d`
- `cargo deny --version`
- `just unused`
- `just deny`
- `cargo audit`
- `cargo metadata --no-deps --format-version 1`
- manifest and lockfile grep checks for Git or registry overrides

Accepted limitations:
- Duplicate `syn` versions remain because they are imposed by incompatible upstream proc-macro stacks and do not create a release-blocking source, license, or advisory issue.
<!-- SECTION:FINAL_SUMMARY:END -->

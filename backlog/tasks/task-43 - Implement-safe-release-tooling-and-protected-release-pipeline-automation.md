---
id: TASK-43
title: Implement safe release tooling and protected release-pipeline automation
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:47'
updated_date: '2026-08-07 14:23'
labels:
  - release
  - ci
  - tooling
milestone: m-5
dependencies:
  - TASK-44
  - TASK-45
  - TASK-46
  - TASK-47
  - TASK-48
  - TASK-49
  - TASK-50
  - TASK-51
priority: high
ordinal: 9000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add deterministic release verification tooling and GitLab CI automation for the v0.2.0 line without allowing accidental production publication. Scope includes version and tag consistency checks, internal dependency validation, package verification commands, release-shape fixture commands, compatibility and documentation release validation, encoded publication order, explicit nonpublishing default modes, protected manual publication flow, artifact retention, and tag-aware release job preparation for v0.2.0-rc.1 and v0.2.0.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Release tooling validates workspace versions, internal dependency versions, and tag or version consistency with actionable failures
- [x] #2 Default release commands do not publish and production publication requires explicit manual authorization with protected credentials
- [x] #3 Release-equivalent dry-run commands cover package verification, package-consumer fixtures, compatibility checks, documentation builds, and artifact assembly
- [x] #4 GitLab CI contains a release-verification path that retains package and documentation artifacts and a protected manual publish path that ordinary branches, merge requests, unprotected tags, and development versions cannot use
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing release-facing command surface in `justfile`, `scripts/release-verify.sh`, `scripts/release-publish.sh`, `.gitlab-ci.yml`, package metadata, and the supporting package, consumer, API, and docs validation commands to confirm the intended dry-run and protected-publish behavior.
2. Run focused positive and negative-path validation for the release tooling: syntax checks, tag-version mismatch failure, publish authorization failure, protected-ref and credential gating, development-version rejection, internal dependency alignment, and the full nonpublishing `just release-verify` flow that assembles retained artifacts.
3. If validation exposes gaps, make the smallest script, CI, or command-surface changes needed to satisfy the release-engineering acceptance criteria, then rerun the affected checks.
4. Record the verification evidence in Backlog and finalize the task only after the release tooling, protected pipeline behavior, and retained dry-run artifacts are all demonstrated.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added nonpublishing release command surfaces in `justfile` (`release-verify` and `release-publish`), plus `scripts/release-verify.sh` and `scripts/release-publish.sh` for version and tag checks, internal dependency alignment checks, dry-run package validation, retained release artifact assembly, and explicit publish authorization via `VVM_RELEASE_PUBLISH=1`.

Extended GitLab CI with a retained-artifact `release-verify` job gated to protected `v*` tags and a separate protected manual `release-publish` job that cannot run from ordinary branches, merge requests, or unprotected tags.

During final release-path validation, tightened `scripts/release-verify.sh` so local dry runs tolerate a normal dirty worktree, package crates in publication order without depending on unpublished crates.io versions, verify extracted package contents from `/tmp/opencode/vvm-release-package-extracted/0.2.0`, and serialize release-validation coverage builds for local stability.

Focused negative-path evidence: `bash scripts/release-verify.sh v0.0.0` rejected a mismatched tag; `bash scripts/release-publish.sh` rejected missing manual authorization; `VVM_RELEASE_PUBLISH=1 CI_COMMIT_TAG=v0.2.0 bash scripts/release-publish.sh` rejected an unprotected ref; `VVM_RELEASE_PUBLISH=1 CI_COMMIT_TAG=v0.2.0 CI_COMMIT_REF_PROTECTED=true bash scripts/release-publish.sh` rejected missing credentials; a minimal temporary repo with `version = 0.2.0-dev` rejected development-version publish attempts; and a minimal temporary repo with a mismatched `vvm-build` dependency version rejected internal dependency drift.

Validated the full nonpublishing release flow with `just release-verify v0.2.0`, which now retains release artifacts under `target/vvm-release/0.2.0`.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented and validated the safe `v0.2.0` release-tooling surface. The repository now provides nonpublishing release verification through `just release-verify` and protected publish gating through `just release-publish`, backed by `scripts/release-verify.sh`, `scripts/release-publish.sh`, and protected GitLab release jobs. The final release verification flow records package file lists, packages crates without assuming crates.io already contains unpublished internal dependencies, simulates publication order through extracted-package checks outside the workspace, retains `target/vvm-release/0.2.0` artifacts, and serializes the release-validation coverage build path for local stability. Negative-path validation confirmed the intended guards for mismatched tags, missing manual authorization, unprotected refs, missing credentials, development-version publish attempts, and internal dependency mismatches.
<!-- SECTION:FINAL_SUMMARY:END -->

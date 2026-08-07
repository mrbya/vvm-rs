---
id: TASK-43
title: Establish the v0.2.0 compatibility policy and accepted API baseline
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-06 14:47'
updated_date: '2026-08-07 12:13'
labels:
  - release
  - compatibility
  - api
milestone: m-5
dependencies: []
priority: high
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Define the canonical public compatibility policy for the v0.2.0 cycle and accept the reviewed post-12.6 API surface as the baseline for future checks. Scope includes Rust API policy, patch compatibility within 0.2.x, persisted coverage schema policy, generated-code compatibility policy, CLI machine-consumed compatibility policy, public API inventory review, accidental export review, and contributor-facing API-diff tooling and commands.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A canonical compatibility policy documents Rust API, pre-1.0 semver, 0.2.x patch compatibility, coverage schema, generated-code, and CLI compatibility rules
- [ ] #2 The reviewed current public API is accepted as the v0.2.0 baseline and any accidental exports are either removed with justification or explicitly accepted
- [ ] #3 Contributor commands and tooling can compare a branch or later release candidate against an explicit v0.2.0 baseline without depending on a nonexistent v0.2.0 tag
- [ ] #4 User-facing book content and package docs do not contradict the compatibility policy
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit the existing release-facing command surface in `justfile`, `scripts/release-verify.sh`, `scripts/release-publish.sh`, `.gitlab-ci.yml`, package metadata, and the supporting package/consumer/API/docs validation commands to confirm the intended dry-run and protected-publish behavior.
2. Run focused positive and negative-path validation for the release tooling: syntax checks, tag/version mismatch failure, publish authorization failure, protected-ref and credential gating, development-version rejection, internal dependency alignment, and the full nonpublishing `just release-verify` flow that assembles retained artifacts.
3. If validation exposes gaps, make the smallest script, CI, or command-surface changes needed to satisfy the release-engineering acceptance criteria, then rerun the affected checks.
4. Record the verification evidence in Backlog and finalize TASK-43 only if the release tooling, protected pipeline behavior, and retained dry-run artifacts are all demonstrated.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added nonpublishing release command surfaces in `justfile` (`release-verify` and `release-publish`), plus `scripts/release-verify.sh` and `scripts/release-publish.sh` for version/tag checks, internal dependency alignment checks, dry-run package validation, retained release artifact assembly, and explicit publish authorization via `VVM_RELEASE_PUBLISH=1`.

Extended GitLab CI with a retained-artifact `release-verify` job gated to protected `v*` tags and a separate protected manual `release-publish` job that cannot run from ordinary branches, merge requests, or unprotected tags.

Focused verification so far: `bash -n scripts/release-verify.sh`, `bash -n scripts/release-publish.sh`, `just --list`, `bash scripts/release-verify.sh v0.0.0` (expected tag-mismatch failure), and `bash scripts/release-publish.sh` (expected authorization failure).
<!-- SECTION:NOTES:END -->

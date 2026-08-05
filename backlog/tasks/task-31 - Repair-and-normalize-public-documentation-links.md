---
id: TASK-31
title: Repair and normalize public documentation links
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
updated_date: '2026-08-05 10:04'
labels: []
milestone: m-3
dependencies:
  - TASK-27
documentation:
  - docs/dev/implementation-plan.md
priority: high
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Audit and repair public documentation links across the book, root README, package READMEs, example READMEs, and public rustdoc comments. Normalize links toward generated public surfaces, remove raw internal-doc links, and validate anchors against rendered output.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every internal book link resolves in rendered output
- [x] #2 Every public book anchor resolves in rendered output
- [x] #3 Every public book-to-rustdoc link resolves against generated API output
- [x] #4 No public book page links to raw Markdown under docs/ or docs/dev/
- [x] #5 Package README and example README documentation links resolve
- [x] #6 No important public information becomes inaccessible while links are normalized
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Repair live public links in the book, README, and rustdoc comments so no rendered page depends on hidden overview chapters, deleted source chapters, or raw docs/dev Markdown paths.
2. Make the public coverage-schema contract reachable from the generated documentation surface, then update all book links to target that rendered location instead of raw docs/ Markdown.
3. Rebuild the documentation outputs needed for focused validation, inspect the rendered targets and anchors, and update the task notes with any redirect or rustdoc path adjustments before finalizing.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Repaired public book navigation links so indexed chapters now point directly to indexed destinations instead of hidden overview pages. Introduction and Quick Start now send readers to `concepts/verification-workflow.md` and `guide/project-setup.md`.

Removed raw maintainer-document links from the public Development chapters by replacing them with local explanatory prose about rustdoc guidance, testing policy, example expectations, and crate-local diagnostics ownership.

Published the coverage artifact schema through the generated book by adding indexed chapter `docs/book/src/development/coverage-json-schema-v1.md`, sourced from `docs/coverage-json-v1.md` with an mdBook include, then retargeted `guide/functional-coverage.md` and the hidden compatibility page `coverage.md` to that rendered chapter.

Normalized stale public links in crate docs by updating `crates/vvm-build/src/lib.rs`, `crates/vvm-macros/src/lib.rs`, and `crates/vvm/README.md` away from the compatibility-only `user-guide.html` path. Removed the raw internal repository-path mention from `README.md`.

Reopened after the regression-check design exposed another public-link class: some book pages still use relative links that escape the published site into repository Markdown or source files, which work in the repository tree but fail on the generated Pages site. This task remains open until those links are normalized to valid public targets.

Normalized one remaining Pages-invalid repository escape in `docs/book/src/guide/using-cargo-vvm.md` by removing the relative `crates/cargo-vvm/README.md` link from the public book. The Guide chapter already covers the intended public workflow, so no information was lost.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Inspected the public book pages, Development chapters, root README, crate READMEs, and public rustdoc crate docs that still referenced hidden overview pages, raw maintainer documents, compatibility-only URLs, or repository-relative Markdown outside the published site. Changed `docs/book/src/introduction.md`, `docs/book/src/quick-start.md`, `docs/book/src/guide/functional-coverage.md`, `docs/book/src/guide/using-cargo-vvm.md`, `docs/book/src/development/contributing.md`, `docs/book/src/development/architecture.md`, `docs/book/src/concepts/overview.md`, `docs/book/src/api-guide/overview.md`, `docs/book/src/guide/user-guide.md`, `docs/book/src/SUMMARY.md`, `README.md`, `crates/vvm-build/src/lib.rs`, `crates/vvm-macros/src/lib.rs`, and `crates/vvm/README.md`; added indexed rendered source `docs/book/src/development/coverage-json-schema-v1.md`; and retained `docs/coverage-json-v1.md` as the source-of-truth content now rendered through the book. Repaired links away from `concepts/overview.md`, `api-guide/overview.md`, `guide/user-guide.md`, `docs/dev/*`, the broken `coverage/schema.md` target, the compatibility-only `user-guide.html` path in crate docs, and the repository-relative `crates/cargo-vvm/README.md` link that would fail on Pages. Focused validation included `just docs-book`, `just docs-api`, repeated `bash scripts/doc-suite.sh` and `bash scripts/check-book-api-links.sh` runs, a grep audit confirming no public book page or README still points at raw `docs/` or `docs/dev/` Markdown or escaped repository Markdown, and explicit `test -s` checks for the README-targeted published pages plus `public/development/coverage-json-schema-v1.html`. Result: the generated public documentation surface now uses canonical book/API links, exposes the coverage-schema contract through the rendered site, and no longer depends on raw internal docs or repository-only Markdown paths.
<!-- SECTION:FINAL_SUMMARY:END -->

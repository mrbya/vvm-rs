---
id: TASK-29
title: Remove orphaned book-source documents
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
updated_date: '2026-08-05 10:03'
labels: []
milestone: m-3
dependencies:
  - TASK-27
documentation:
  - docs/book/src/SUMMARY.md
  - docs/dev/implementation-plan.md
priority: high
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Delete obsolete chapter-like Markdown sources under docs/book/src once the audit confirms they are not indexed not required as special files not included and not required for compatibility redirects. Preserve redirect behavior through site assembly instead of retaining stale source chapters.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every remaining chapter-like Markdown file under docs/book/src is indexed or explicitly justified as a special source
- [x] #2 Obsolete source pages superseded by indexed chapters are removed
- [x] #3 Empty legacy directories under docs/book/src are removed
- [x] #4 Stale links includes and comments referring to deleted source pages are removed or updated
- [x] #5 Compatibility redirects continue to resolve after source deletions
- [x] #6 The book builds successfully after orphan cleanup
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Update any compatibility redirect that still points at a hidden source chapter so the assembled site depends only on indexed destinations.
2. Delete the obsolete hidden overview, coverage, cargo-vvm, and reference source files identified in the audit, then remove any now-empty legacy directories.
3. Rebuild the book and assembled site, confirm no repository source still points at deleted chapters, and verify that compatibility redirects still land on existing rendered pages and anchors.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Repointed `scripts/doc-suite.sh` so the preserved `public/user-guide.html` compatibility URL now lands on indexed chapter `guide/project-setup.html` instead of deleted hidden source `guide/user-guide.html`.

Removed the obsolete hidden chapter-like sources identified in the audit: `docs/book/src/concepts/overview.md`, `docs/book/src/api-guide/overview.md`, `docs/book/src/guide/user-guide.md`, `docs/book/src/coverage.md`, `docs/book/src/cargo-vvm.md`, `docs/book/src/reference.md`, and the legacy fragmented `docs/book/src/cargo-vvm/*.md` source set.

Removed the empty legacy directory `docs/book/src/cargo-vvm/` after deleting its chapter files. Post-cleanup inventory shows `60` Markdown files under `docs/book/src`, `58` indexed by `SUMMARY.md`, and only `docs/book/src/SUMMARY.md` plus `docs/book/src/404.md` remaining unindexed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Deleted the orphaned hidden book sources that were no longer indexed, included, or needed for compatibility redirects. Removed `docs/book/src/concepts/overview.md`, `docs/book/src/api-guide/overview.md`, `docs/book/src/guide/user-guide.md`, `docs/book/src/coverage.md`, `docs/book/src/cargo-vvm.md`, `docs/book/src/reference.md`, `docs/book/src/cargo-vvm/installation.md`, `docs/book/src/cargo-vvm/workflow.md`, `docs/book/src/cargo-vvm/command-reference.md`, `docs/book/src/cargo-vvm/output-layout.md`, `docs/book/src/cargo-vvm/merge-policies.md`, `docs/book/src/cargo-vvm/failure-semantics.md`, `docs/book/src/cargo-vvm/gitlab.md`, and `docs/book/src/cargo-vvm/troubleshooting.md`, then removed the emptied `docs/book/src/cargo-vvm/` directory. Updated `scripts/doc-suite.sh` so the retained `user-guide.html` compatibility redirect now targets `guide/project-setup.html`, which is indexed and rendered without depending on obsolete source chapters. Validation included `just docs-book`, `bash scripts/doc-suite.sh`, precise grep checks confirming no repository source still points at the deleted chapter paths, explicit `test -s` checks for representative compatibility outputs, and a post-cleanup inventory check confirming that only `docs/book/src/SUMMARY.md` and `docs/book/src/404.md` remain unindexed under `docs/book/src`. Compatibility redirects still build successfully after deletion, and the book source tree no longer contains obsolete overview, cargo-vvm, coverage, or reference chapters.
<!-- SECTION:FINAL_SUMMARY:END -->

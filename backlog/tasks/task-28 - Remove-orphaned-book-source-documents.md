---
id: TASK-28
title: Remove orphaned book-source documents
status: To Do
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
labels: []
milestone: m-3
dependencies: []
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
- [ ] #1 Every remaining chapter-like Markdown file under docs/book/src is indexed or explicitly justified as a special source
- [ ] #2 Obsolete source pages superseded by indexed chapters are removed
- [ ] #3 Empty legacy directories under docs/book/src are removed
- [ ] #4 Stale links includes and comments referring to deleted source pages are removed or updated
- [ ] #5 Compatibility redirects continue to resolve after source deletions
- [ ] #6 The book builds successfully after orphan cleanup
<!-- AC:END -->

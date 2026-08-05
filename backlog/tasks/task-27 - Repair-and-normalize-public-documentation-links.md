---
id: TASK-27
title: Repair and normalize public documentation links
status: To Do
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
labels: []
milestone: m-3
dependencies: []
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
- [ ] #1 Every internal book link resolves in rendered output
- [ ] #2 Every public book anchor resolves in rendered output
- [ ] #3 Every public book-to-rustdoc link resolves against generated API output
- [ ] #4 No public book page links to raw Markdown under docs/ or docs/dev/
- [ ] #5 Package README and example README documentation links resolve
- [ ] #6 No important public information becomes inaccessible while links are normalized
<!-- AC:END -->

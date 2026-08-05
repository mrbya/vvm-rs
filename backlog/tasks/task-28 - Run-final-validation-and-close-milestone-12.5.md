---
id: TASK-28
title: Run final validation and close milestone 12.5
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
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
From a clean documentation output state, run the complete documentation and repository validation suites, verify the published site entry points and redirects, reconcile docs/dev/implementation-plan.md with Backlog, add a final milestone summary, and close milestone 12.5 truthfully without starting 12.6 work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Documentation commands pass from a clean output state including docs-book docs-api docs-internal docs-test docs-links and docs-site
- [ ] #2 Required published site files exist under public/ including index 404 quick-start guide development and api entry points
- [ ] #3 Every generated compatibility redirect file exists and its destination page and destination anchor resolve beneath the Pages base path
- [ ] #4 The final authoritative list of docs/book/src Markdown files shows only indexed chapters plus explicitly justified special files
- [ ] #5 The full repository validation suite passes without weakening checks
- [ ] #6 All milestone 12.5 cleanup tasks are terminal with plans final summaries and completed acceptance criteria
- [ ] #7 docs/dev/implementation-plan.md agrees with Backlog state and milestone 12.5 is closed without starting 12.6 work
<!-- AC:END -->

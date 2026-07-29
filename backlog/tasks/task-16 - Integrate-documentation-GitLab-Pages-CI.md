---
id: TASK-16
title: Integrate documentation GitLab Pages CI
status: Done
assignee:
  - OpenCode
created_date: '2026-07-29 14:12'
updated_date: '2026-07-29 14:22'
labels: []
milestone: m-0
dependencies: []
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add docs tests site artifact Pages deployment and deployment smoke checks using current GitLab Pages syntax.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Documentation tests and site build run in CI
- [x] #2 Pages publishing is limited to intended refs
- [x] #3 Pages smoke checks cover book API and cargo-vvm URLs
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Add GitLab documentation test, site artifact, Pages publish, and output smoke jobs using current pages.publish syntax.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added documentation-tests, documentation-site, deploy-pages, and pages-smoke jobs. Pages uses the current `pages: publish: public` syntax and deploys only from the default branch.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-28
title: Add documentation hygiene regression checks
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
updated_date: '2026-08-05 10:10'
labels: []
milestone: m-3
dependencies:
  - TASK-27
documentation:
  - docs/dev/implementation-plan.md
priority: high
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the existing documentation validation scripts and commands so broken internal links anchors rustdoc links redirects missing SUMMARY targets and orphaned book chapters fail locally and in CI with actionable diagnostics.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Missing SUMMARY targets fail the documentation hygiene check
- [x] #2 Orphaned book chapters fail the documentation hygiene check
- [x] #3 Forbidden links from public book pages to raw docs/ or docs/dev/ Markdown fail the documentation hygiene check
- [x] #4 Broken rendered-book links anchors rustdoc links and redirect targets fail the documentation hygiene check
- [x] #5 Existing contributor-facing documentation commands remain the entry point for the checks
- [x] #6 Diagnostics identify the source file and broken or forbidden target clearly
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace the broken `docs-links` wiring with a real `docs-suite` recipe and one documentation hygiene script that validates SUMMARY targets, orphaned chapters, forbidden raw-doc links, rendered anchors, rustdoc paths, and redirect destinations.
2. Make the checker operate on the assembled `public/` site so failures reflect the published output, while still reporting the source markdown file and target that caused the failure.
3. Integrate the checker into the existing `just` documentation commands, then rerun the focused docs command set until the diagnostics and behavior are stable.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Added `scripts/check-doc-hygiene.sh` as the assembled-site documentation checker. It validates SUMMARY targets, orphaned chapters, forbidden relative markdown links that escape the book, rendered book and rustdoc HTML targets, rendered anchors, redirect destinations, redirect anchors, and the `public/404.html` Pages base path.

Updated `docs/book/book.toml` `site-url` from `/` to `/vvm-rs/` so the generated 404 page now carries `<base href="/vvm-rs/">`, matching the GitLab Pages project base path.

Added `docs-suite` and `docs-site` recipes to the `justfile`, pointed `docs` at `docs-suite`, switched `docs-links` to the new hygiene checker, and integrated `docs-links` into `just ci` so documentation hygiene failures block the CI-equivalent command.

Kept `scripts/check-book-api-links.sh` as a thin wrapper to `scripts/check-doc-hygiene.sh` so existing local habits and references still reach the new validation logic.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Extended the documentation quality gate from a narrow API-link check into an assembled-site hygiene checker. Added `scripts/check-doc-hygiene.sh` to validate missing `SUMMARY.md` targets, orphaned `docs/book/src` chapters, forbidden raw `docs/` and `docs/dev/` markdown links from indexed chapters, Pages-invalid relative markdown links, broken rendered book links and anchors, broken book-to-rustdoc links, broken redirect destinations and anchors, and the published `404.html` base-path configuration. Updated `docs/book/book.toml` so the book uses `site-url = "/vvm-rs/"`, which fixes the generated 404 base for GitLab Pages project hosting. Updated `justfile` to add `docs-suite` and `docs-site`, route `docs` through `docs-suite`, run `scripts/check-doc-hygiene.sh` from `docs-links`, and include `docs-links` in `just ci` so merge requests fail on documentation hygiene regressions. Retained `scripts/check-book-api-links.sh` as a compatibility wrapper around the new checker. Validation for this task used `just docs-test`, `just docs-site`, and `just docs-links`, all passing after the new checker and base-path fix. The checker emits source-and-target diagnostics such as missing SUMMARY targets, orphaned chapters, forbidden internal-doc links, Pages-invalid relative markdown links, broken rendered anchors, and broken redirect anchors.
<!-- SECTION:FINAL_SUMMARY:END -->

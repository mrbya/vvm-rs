---
id: TASK-30
title: Run final validation and close milestone 12.5
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
updated_date: '2026-08-05 10:51'
labels: []
milestone: m-3
dependencies:
  - TASK-28
  - TASK-29
  - TASK-27
  - TASK-31
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
- [x] #1 Documentation commands pass from a clean output state including docs-book docs-api docs-internal docs-test docs-links and docs-site
- [x] #2 Required published site files exist under public/ including index 404 quick-start guide development and api entry points
- [x] #3 Every generated compatibility redirect file exists and its destination page and destination anchor resolve beneath the Pages base path
- [x] #4 The final authoritative list of docs/book/src Markdown files shows only indexed chapters plus explicitly justified special files
- [x] #5 The full repository validation suite passes without weakening checks
- [x] #6 All milestone 12.5 cleanup tasks are terminal with plans final summaries and completed acceptance criteria
- [x] #7 docs/dev/implementation-plan.md agrees with Backlog state and milestone 12.5 is closed without starting 12.6 work
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Run the full documentation validation sequence from a clean output state, including docs-clean, docs-book, docs-api, docs-internal, docs-test, docs-links, docs-site, explicit published-file assertions, and redirect checks.
2. Run the full repository validation suite without weakening checks, fix any regressions, and rerun until the entire gate passes.
3. Reconcile docs/dev/implementation-plan.md with the completed Backlog state, confirm all milestone tasks are terminal with plans and summaries, archive the closing-pass milestone, and then finalize this task with the complete validation record.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Ran the clean documentation validation sequence: `just docs-clean`, `just docs-book`, `just docs-api`, `just docs-internal`, `just docs-test`, `just docs-links`, and `just docs-site`. Verified required published outputs with explicit `test -s` checks for `public/index.html`, `public/404.html`, `public/quick-start.html`, `public/guide/functional-coverage.html`, `public/guide/using-cargo-vvm.html`, `public/guide/generated-type-mapping.html`, `public/guide/execution-order.html`, `public/guide/troubleshooting.html`, `public/guide/compatibility-and-limitations.html`, `public/development/contributing.html`, `public/development/architecture.html`, `public/api/index.html`, `public/api/vvm/index.html`, `public/api/vvm_build/index.html`, `public/api/vvm_core/index.html`, `public/api/vvm_macros/index.html`, and `public/build-info.json`.

Redirect validation passed through `scripts/check-doc-hygiene.sh`, which checks every `write_redirect` entry in `scripts/doc-suite.sh` for generated-file existence, destination-page existence, and destination-anchor existence under the assembled Pages layout. Representative preserved outputs verified explicitly include `public/getting-started.html`, `public/user-guide.html`, `public/coverage.html`, `public/cargo-vvm.html`, `public/cargo-vvm/workflow.html`, `public/reference.html`, and `public/reference/execution-order.html`.

Confirmed the final book-source inventory after cleanup: `60` Markdown files under `docs/book/src`, `58` indexed by `SUMMARY.md`, and only `docs/book/src/SUMMARY.md` plus `docs/book/src/404.md` retained unindexed. Updated `docs/dev/implementation-plan.md` so milestone 12 reflects `12.5` complete, `12.6` not started, the current mdBook layout, the hidden-source cleanup, the redirect strategy, and the rendered coverage-schema chapter.

Ran the full repository validation suite without weakening checks: `just fmt --check`, `just check -- -D warnings`, `just test-fast`, `just test-native-fixtures`, `just test-examples`, `just test-native`, `just test-e2e`, `just test-package`, `just test-all`, `just doctest`, `just test-cov-ci`, `just unused`, `just audit`, and `just ci`. The final `just ci` run now includes the documentation hygiene gate and passed.

Archived milestone `12.5 closing hygiene pass` (`m-3`) after the validation gates passed. Backlog audit at closure time: `TASK-27`, `TASK-28`, `TASK-29`, `TASK-30`, and `TASK-31` all have recorded plans, acceptance criteria marked complete, final summaries, and terminal states; no benchmark or milestone `12.6` Backlog task was started.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Completed the final milestone 12.5 documentation hygiene closure. From a clean output state, ran `just docs-clean`, `just docs-book`, `just docs-api`, `just docs-internal`, `just docs-test`, `just docs-links`, and `just docs-site`, then verified the required published entry points and API indexes under `public/`. Redirect validation covered every generated compatibility page from `scripts/doc-suite.sh`; preserved historical paths such as `getting-started.html`, `user-guide.html`, `coverage.html`, `coverage/*`, `cargo-vvm.html`, `cargo-vvm/*`, `reference.html`, and `reference/*` all resolved to existing destination pages and anchors beneath the GitLab Pages base path. Confirmed the authoritative book-source inventory now contains `60` Markdown files under `docs/book/src`, with `58` indexed chapters and only `docs/book/src/SUMMARY.md` plus `docs/book/src/404.md` intentionally unindexed. Updated `docs/dev/implementation-plan.md` to match the real 12.5 end state: the hidden compatibility chapters are gone, the public coverage schema is rendered through the book, 12.5 is complete, and 12.6 benchmark work has not started. Ran the full repository validation suite: `just fmt --check`, `just check -- -D warnings`, `just test-fast`, `just test-native-fixtures`, `just test-examples`, `just test-native`, `just test-e2e`, `just test-package`, `just test-all`, `just doctest`, `just test-cov-ci`, `just unused`, `just audit`, and `just ci`; all passed. Closed the Backlog side truthfully by confirming every 12.5 closing-pass task (`TASK-27`, `TASK-28`, `TASK-29`, `TASK-30`, `TASK-31`) has a plan, completed acceptance criteria, a final summary, and a terminal state, then archived milestone `12.5 closing hygiene pass` (`m-3`). No milestone 12.6 Backlog task was started.
<!-- SECTION:FINAL_SUMMARY:END -->

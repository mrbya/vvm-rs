---
id: TASK-22
title: Write the public API usage guide for the VVM facade and direct-entry crates
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-08-04 14:51'
labels: []
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create a usage-oriented API guide that inventories the major public facade subsystems and explains how to use them correctly without forcing readers to infer workflows from rustdoc or examples alone. Cover generated DUT integration, build APIs, derives, test registration, testbench composition, randomization, tracing, timing, inout behavior, coverage, configuration, and errors, while linking to exact rustdoc types and methods.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every API Guide rustdoc link resolves in the assembled public site
- [x] #2 Public re-exports link through the intended `vvm` facade when that is the user entry point
- [x] #3 Macro derive attribute and method links resolve to the correct rustdoc pages
- [x] #4 Link validation for API Guide rustdoc targets is automated as part of documentation validation
- [x] #5 The API Guide remains usage-focused instead of being rewritten without need
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Assemble the public site and audit every rustdoc-facing link in `docs/book/src/api-guide/` against the generated paths under `public/api/`.
2. Prefer stable `vvm` facade links for re-exported user entry points, and use direct crate links only when the chapter intentionally documents a direct-entry crate.
3. Fix macro, derive, attribute, method, and re-export path mistakes, then normalize link style where it improves maintainability.
4. Add automated validation for these links by checking the generated site paths that the API Guide depends on.
5. Rebuild docs-site and rerun link validation after the audit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Reopened for the final 12.5 API Guide pass to audit and repair every rustdoc link against the assembled public site while keeping the API Guide usage-focused rather than expanding prose unnecessarily.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Audited and repaired the API Guide’s rustdoc links against the assembled `public/api/` site. Updated the API Guide chapters to link to the intended public `vvm` facade items for the user entry points, including `attr.test.html`, derive pages, specific module items, struct pages, trait pages, enum pages, and method anchors for `DutBuilder`, `Testbench`, `TestRunConfig`, `TestContext`, schedulers, coverage, reports, and errors. Added automated assembled-site validation for API Guide rustdoc paths via `scripts/check-book-api-links.sh`, which now runs from `just docs-links`. Important decision: kept the API Guide usage-focused and only deepened links, not prose, unless a specific item-level rustdoc target was needed. Validation: `just docs-links`, `just docs-site`, explicit public API entry-point checks, and the full repository gate including `just ci` passed.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: TASK-27
title: Audit the documentation link graph and book-source inventory
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-05 09:54'
updated_date: '2026-08-05 09:58'
labels: []
milestone: m-3
dependencies: []
documentation:
  - docs/dev/implementation-plan.md
  - docs/book/src/SUMMARY.md
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Perform the closing hygiene audit for milestone 12.5. Build a complete inventory of mdBook sources, SUMMARY targets, special sources, includes, assets, redirects, and public documentation links so the repository has a truthful repair and deletion plan before changing documentation structure.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every Markdown file under docs/book/src is classified as indexed chapter SUMMARY special source include or asset obsolete compatibility source or needs investigation
- [x] #2 Every SUMMARY.md target is verified to exist
- [x] #3 Every unindexed book source is either justified or marked for removal
- [x] #4 Every link to docs/ or docs/dev/ from public documentation is identified
- [x] #5 Every compatibility redirect and inbound legacy path is inventoried
- [x] #6 A concrete repair and deletion plan is recorded in task notes or a development-only record
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Inventory every Markdown and static source under docs/book/src and compare the tree against docs/book/src/SUMMARY.md, mdBook special files, include fragments, and generated compatibility redirects.
2. Build the public-link map across docs/book/src, README.md, crates/*/README.md, examples/*/README.md, and public rustdoc comments to identify raw internal-doc links, stale chapter links, anchor risks, and rustdoc targets that require follow-up repair.
3. Record the audit matrix with classification, inbound links, redirect status, replacement targets, and planned action in the task notes so the cleanup, link-repair, and regression-check tasks can execute against one authoritative inventory.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audit inventory for milestone 12.5 closing hygiene pass.

Book-source counts:
- 73 Markdown files under docs/book/src.
- 57 SUMMARY targets; all 57 exist.
- 35 mdBook include directives from indexed chapters, all pulling source snippets from tests/fixtures or example fixture files outside docs/book/src.
- No image assets or static source fragments live under docs/book/src today.

Classification:
- Indexed chapters: every Markdown file referenced by docs/book/src/SUMMARY.md (57 files).
- Special unindexed sources that remain justified: docs/book/src/SUMMARY.md and docs/book/src/404.md.
- Obsolete hidden chapter-like sources that should be removed or replaced with redirects only: docs/book/src/concepts/overview.md, docs/book/src/api-guide/overview.md, docs/book/src/guide/user-guide.md, docs/book/src/coverage.md, docs/book/src/cargo-vvm.md, docs/book/src/reference.md, docs/book/src/cargo-vvm/installation.md, docs/book/src/cargo-vvm/workflow.md, docs/book/src/cargo-vvm/command-reference.md, docs/book/src/cargo-vvm/output-layout.md, docs/book/src/cargo-vvm/merge-policies.md, docs/book/src/cargo-vvm/failure-semantics.md, docs/book/src/cargo-vvm/gitlab.md, docs/book/src/cargo-vvm/troubleshooting.md.

Known inbound links to obsolete hidden sources:
- introduction.md and quick-start.md link to concepts/overview.md and guide/user-guide.md.
- concepts/overview.md links to api-guide/overview.md.
- api-guide/overview.md links to guide/user-guide.md.
- guide/user-guide.md links back to concepts/overview.md and api-guide/overview.md.
- coverage.md links to missing coverage/schema.md.

Public raw-internal-doc links found in the book:
- development/contributing.md -> docs/dev/rustdoc_style.md
- development/contributing.md -> docs/dev/testing-strategy.md
- development/contributing.md -> docs/dev/example-strategy.md
- development/architecture.md -> docs/dev/errors-and-diagnostics.md

Other public-surface findings:
- guide/functional-coverage.md links to ../../../coverage-json-v1.md, which escapes the book to a raw docs file and will fail on the assembled Pages site because public/ does not publish that Markdown path.
- README.md mentions docs/dev/reference-projects/vvm/ as a raw internal repository path in a public section.
- Public rustdoc crate docs still link users to https://byacrates.gitlab.io/vvm-rs/user-guide.html from crates/vvm-build/src/lib.rs and crates/vvm-macros/src/lib.rs; that compatibility URL should remain valid but no obsolete source chapter is needed to support it.

Compatibility redirect inventory from scripts/doc-suite.sh:
- getting-started.html -> quick-start.html
- user-guide.html -> guide/user-guide.html
- coverage.html -> guide/functional-coverage.html
- coverage/bins.html -> guide/functional-coverage.html#bins
- coverage/coverpoints.html -> guide/functional-coverage.html#coverpoints
- coverage/typed-models.html -> guide/functional-coverage.html#typed-coverage-models
- coverage/crosses.html -> guide/functional-coverage.html#cross-coverage
- coverage/sampling.html -> guide/functional-coverage.html#sampling-and-observedcycle
- coverage/sessions-and-artifacts.html -> guide/functional-coverage.html#per-test-artifacts-and-snapshots
- coverage/merging.html -> guide/functional-coverage.html#merge-compatibility
- coverage/reporting.html -> guide/functional-coverage.html#reports
- coverage/ci.html -> guide/functional-coverage.html#ci-integration
- cargo-vvm.html -> guide/using-cargo-vvm.html
- cargo-vvm/installation.html -> guide/using-cargo-vvm.html#installation
- cargo-vvm/workflow.html -> guide/using-cargo-vvm.html#normal-workflow
- cargo-vvm/command-reference.html -> guide/using-cargo-vvm.html#command-summary
- cargo-vvm/output-layout.html -> guide/using-cargo-vvm.html#output-directory-and-layout
- cargo-vvm/merge-policies.html -> guide/using-cargo-vvm.html#merge-policies
- cargo-vvm/failure-semantics.html -> guide/using-cargo-vvm.html#failure-semantics
- cargo-vvm/gitlab.html -> guide/using-cargo-vvm.html#gitlab-ci-example
- cargo-vvm/troubleshooting.html -> guide/using-cargo-vvm.html#troubleshooting
- reference.html -> guide/configuring-tests.html
- reference/configuration.html -> guide/configuring-tests.html#configuration-layers
- reference/environment-variables.html -> guide/configuring-tests.html#environment-variables
- reference/generated-types.html -> guide/generated-type-mapping.html
- reference/execution-order.html -> guide/execution-order.html
- reference/artifact-layout.html -> guide/using-cargo-vvm.html#output-directory-and-layout
- reference/compatibility.html -> guide/compatibility-and-limitations.html
- reference/diagnostics.html -> guide/troubleshooting.html
- reference/terminology.html -> concepts/verification-workflow.html
- reference/limitations.html -> guide/compatibility-and-limitations.html

Immediate repair plan:
1. Replace all live links to obsolete hidden overview chapters with indexed chapter targets, then repoint compatibility redirects away from obsolete source chapters.
2. Remove raw links from public book pages to docs/dev/*.md and to docs/coverage-json-v1.md by moving or rendering the needed public contract into the generated site.
3. Delete the 14 obsolete hidden chapter-like Markdown files and empty legacy cargo-vvm directory after redirects and inbound links are repaired.
4. Replace the broken docs-links wiring with a real docs-suite / hygiene check that validates SUMMARY targets, orphan chapters, rendered anchors, rustdoc links, and redirect destinations.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Inspected the full `docs/book/src` tree, `docs/book/src/SUMMARY.md`, mdBook include usage, the site assembly redirects in `scripts/doc-suite.sh`, public README surfaces, and public rustdoc crate docs. Audited `73` Markdown book-source files, verified all `57` `SUMMARY.md` targets exist, classified the unindexed files, and recorded the full redirect inventory and repair/deletion matrix in task notes. Identified `14` obsolete hidden chapter-like sources, four raw `docs/dev/*` links from public book pages, one raw `docs/coverage-json-v1.md` link from the book, one broken hidden-source link to `coverage/schema.md`, and a broken contributor command surface where `just docs-links` calls a missing `docs-suite` recipe. Validation for the audit phase used static repository searches plus a `SUMMARY.md` existence check and one run of `just docs-links` to capture the current command failure state. No files were changed in this audit task; the recorded findings drive the follow-on link-repair, orphan-cleanup, regression-check, and final-validation tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

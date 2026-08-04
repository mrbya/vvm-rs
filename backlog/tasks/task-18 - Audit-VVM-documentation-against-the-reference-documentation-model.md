---
id: TASK-18
title: Audit VVM documentation against the reference documentation model
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-07-30 09:27'
updated_date: '2026-07-30 16:10'
labels: []
milestone: Milestone 12.5 — User-centred documentation rewrite
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Inspect the current VVM documentation surfaces against the repository's actual public APIs and examples, using .idea/zappy as a read-only reference for documentation scope, progression, and structure. Record a detailed coverage audit covering the root README, package READMEs, mdBook chapters, example READMEs, public facade modules, major public types and macros, environment variables, contributor workflows, and maintainer strategy documents. Identify shallow chapters, README-only information, API gaps, missing examples, weak navigation, terminology gaps, troubleshooting gaps, and the future authoritative location for each major topic before the rewrite begins.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 .idea/zappy is inspected read-only for documentation principles and scope
- [x] #2 VVM README book package README example README rustdoc and developer-doc coverage is inventoried
- [x] #3 Information present only in the root README or example README files is identified
- [x] #4 Major public API documentation gaps and missing example coverage are identified
- [x] #5 A future authoritative location is assigned to every major documentation topic
- [x] #6 A concrete rewrite plan is recorded in Backlog and the maintainer documentation audit artifact
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Re-audit the current public mdBook, README surfaces, example READMEs, and supporting fixtures against the final user-centred rewrite brief rather than the earlier migration milestone.
2. Record a page-by-page inventory for public book chapters with audience, current purpose, current depth, example dependence, internal-process leakage, missing background, missing code, missing diagnostics, and planned action.
3. Identify which existing milestone 12.5 tasks remain valid, which ones must be reopened, and whether any additional scope must be captured under the same milestone.
4. Update the maintainer audit artifact and task notes so the rest of milestone 12.5 can proceed from a truthful implementation plan.
5. Finalize TASK-18 only after the audit artifact matches the actual repository state and the milestone task breakdown is accurate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Adopted Zappy-inspired principles for VVM: concise but self-sufficient root README; a book organized by orientation, workflows, feature guides, reference, and development material; step-by-step task-oriented chapters; separation of conceptual guidance from exact API reference; explicit documentation of current limitations; detailed crate-boundary and contributor workflow pages; and navigation that carries readers from first project setup to implementation architecture.

Reopened after validating the current public book against the final user-centred rewrite brief. The done state was inaccurate: public pages still leak milestone history, several major guide chapters remain summary-only, and the early reader journey still lacks the required product-positioning and HDL-pipeline chapters.

Updated `docs/dev/documentation-audit.md` with a page-by-page public-book inventory covering orientation, concepts, guide chapters, API guide pages, coverage, examples, cargo-vvm, and reference pages. Each row now records audience, purpose, depth, example dependence, internal-process leakage, missing background, missing code, missing diagnostics, and the planned corrective action.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Revalidated the public documentation against the final user-centred milestone 12.5 rewrite brief and corrected the earlier high-level audit. `docs/dev/documentation-audit.md` now contains a page-by-page inventory of the public book, identifying sparse chapters, example-dependent teaching, internal-process leakage, missing HDL-pipeline context, missing Rust context, missing code, missing diagnostics, and the corrective action for each page. Reopened the prematurely closed milestone 12.5 tasks and recreated an active milestone so the remaining rewrite can proceed from a truthful plan of record.
<!-- SECTION:FINAL_SUMMARY:END -->

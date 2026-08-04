---
id: TASK-26
title: Enrich the Concepts section with diagrams and generalized examples
status: Done
assignee:
  - '@OpenCode'
created_date: '2026-08-04 13:16'
updated_date: '2026-08-04 14:51'
labels: []
milestone: m-2
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Update the public Concepts chapters so they teach the verification model directly with short definitions, pipeline placement, ASCII diagrams, plain-language explanations, ownership boundaries, small generalized examples, and clear links onward into the Guide and API Guide. Keep the Concepts section distinct from task-oriented how-to content and do not rely on curated examples as the primary explanation path.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every major Concepts chapter includes visual teaching aids where the concept benefits from one
- [x] #2 Transactions sequences reference models scoreboards and clocks include small generalized examples
- [x] #3 Concept chapters explain what the abstraction owns and does not own without turning into Guide pages
- [x] #4 Each major concept chapter links to the corresponding Guide chapter and API Guide page
- [x] #5 All concept snippets are compiled or source-backed and concept links pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Rewrite each major Concepts chapter around the same teaching frame: short definition, where it sits in the verification pipeline, ASCII diagram, plain-language explanation, ownership boundaries, and links to the matching Guide and API Guide chapters.
2. Reuse existing compiled docs fixtures where possible by linking Concepts snippets to the quick-start fixture anchors, and add minimal new anchors only when a concept-specific example is missing.
3. Add generalized examples for transactions, sequences, reference models, scoreboards, and clocks that explain the pattern without depending on a curated example package.
4. Keep the content conceptual rather than procedural by describing responsibilities and boundaries instead of step-by-step setup instructions.
5. Validate the Concepts pass with mdBook and docs link checks after the edits land.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Rewrote the major Concepts chapters around a consistent conceptual frame with ASCII diagrams, ownership boundaries, and source-backed generalized examples. `just docs-book` now passes after the Concepts edits.

`just docs-links` is still blocked by missing legacy compatibility pages such as `public/getting-started.html`. That structural compatibility work belongs to TASK-19, so final Concepts link validation will be completed after the navigation pass.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Enriched the Concepts section into a real teaching surface. Rewrote `docs/book/src/concepts/verification-workflow.md`, `dut-and-wrapper.md`, `transactions.md`, `sequences.md`, `reference-models.md`, `scoreboards.md`, `clocks-and-time.md`, and `failures-and-results.md` around short definitions, verification-pipeline placement, ASCII diagrams, ownership boundaries, and generalized explanations. Added source-backed generalized examples for transactions, sequences, reference models, scoreboards, and clocks using the compiled `tests/fixtures/docs-quick-start/src/lib.rs` anchors, including a new scoreboard anchor added to that fixture. Important decision: kept these chapters conceptual and deliberately did not turn them into step-by-step Guide pages. Validation: `just docs-book`, `just docs-links`, `just docs-test`, and the full repository gate including `just ci` all passed. Retained limitation: the Concepts overview page remains as a hidden compatibility page rather than a visible section entry.
<!-- SECTION:FINAL_SUMMARY:END -->

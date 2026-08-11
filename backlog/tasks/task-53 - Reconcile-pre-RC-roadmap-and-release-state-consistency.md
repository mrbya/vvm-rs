---
id: TASK-53
title: Reconcile pre-RC roadmap and release-state consistency
status: Done
assignee:
  - OpenCode
created_date: '2026-08-11 11:57'
updated_date: '2026-08-11 12:02'
labels: []
milestone: m-6
dependencies: []
documentation:
  - docs/dev/implementation-plan.md
  - CHANGELOG.md
  - CONTRIBUTING.md
  - README.md
  - .gitlab-ci.yml
  - scripts/release-verify.sh
  - scripts/release-publish.sh
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Make the roadmap, release metadata, contributor docs, release scripts, and public status statements internally consistent for the v0.2.0 pre-RC state before the technical audit begins. Reconcile any stale milestone 12.7 wording, especially obsolete requirements for standalone RELEASING.md or SECURITY.md files that were intentionally consolidated into CONTRIBUTING.md. Preserve historical v0.1.0-alpha.1 references and do not begin milestone 12.9 work.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Roadmap text matches actual milestone 12.7 decisions
- [x] #2 No stale RELEASING.md requirement remains
- [x] #3 No stale SECURITY.md requirement remains
- [x] #4 v0.2.0 and v0.2.0-rc.1 references are consistent
- [x] #5 Historical v0.1.0-alpha.1 references remain accurate
- [x] #6 No milestone 12.9 work is started
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Audit release-facing status sources: implementation plan, README, CHANGELOG, CONTRIBUTING, package READMEs, CI, and release scripts.
2. Search for stale milestone 12.7 wording, obsolete RELEASING.md or SECURITY.md requirements, and inconsistent v0.2.0 / v0.2.0-rc.1 references while preserving historical v0.1.0-alpha.1 references.
3. Update the smallest set of docs and metadata needed to make the repository’s current pre-RC state internally consistent without starting milestone 12.9 work.
4. Run focused validation on the edited docs and scripts, then check TASK-53 acceptance criteria and record a final summary.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Audited implementation-plan, README, changelog, contributor guide, release scripts, CI, and package READMEs for pre-RC status drift.

Corrected milestone 12 summary text and 12.7/12.8 release-state wording in docs/dev/implementation-plan.md so 12.8 is explicitly the final pre-RC audit and 12.9 remains unstarted.

Removed the stale standalone RELEASING.md and SECURITY.md requirement from the implementation plan by documenting the accepted consolidation into CONTRIBUTING.md.

Updated the top-level README status note so public docs now state the current pre-1.0 v0.2.0 freeze, the 12.8 final-audit state, and the 12.9 RC-publication boundary.

Focused validation used targeted grep and readback checks to confirm no remaining stale roadmap requirement for standalone RELEASING.md or SECURITY.md files outside historical/archive records.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Audited the pre-RC release-state sources and reconciled the repository wording with the accepted milestone 12.7 outcome and the current milestone 12.8 state.

What changed:
- Updated `docs/dev/implementation-plan.md` to mark milestone 12 as having 12.8 in progress and 12.9 still unstarted.
- Corrected the 12.7 status text to describe 12.8 as the final audit before release-candidate freeze.
- Replaced the stale 12.7 requirement for standalone `RELEASING.md` and `SECURITY.md` files with the implemented policy: release and security-reporting guidance is consolidated into `CONTRIBUTING.md`.
- Updated the top-level `README.md` status note to state that VVM remains pre-`1.0`, the `v0.2.0` API is established for the current release cycle, milestone 12.8 is the final audit before RC freeze, and milestone 12.9 will handle `v0.2.0-rc.1` publication and dogfooding.

Findings:
- The roadmap still contained stale milestone 12.7 wording that implied missing standalone release/security documents.
- The roadmap and README did not yet describe the actual current 12.8 versus 12.9 boundary.
- Historical `v0.1.0-alpha.1` references were already accurate and were preserved.

Commands and validation:
- Repository-wide grep for `RELEASING.md`, `SECURITY.md`, `v0.2.0`, `v0.2.0-rc.1`, and milestone references.
- Readback checks on `README.md` and the relevant implementation-plan sections after the edits.

Accepted limitations:
- No additional public-book wording was changed in this task because the top-level roadmap and README were the stale sources; broader documentation hygiene remains in later milestone 12.8 tasks.
<!-- SECTION:FINAL_SUMMARY:END -->

---
id: m-5
title: "Milestone 12.7 — v0.2.0 Packaging, Compatibility, Status Polish, and Release Engineering"
---

## Description

Prepare the VVM workspace for the v0.2.0 release line without publishing production crates or starting milestone 12.8. Covers package metadata and archive validation, compatibility policy and matrix, API/schema/generated-contract baselines, public status updates, changelog and contributing guides, release tooling and CI dry-run automation, full release simulation, and final implementation-plan alignment.

## Summary

Milestone 12.7 is complete for the `v0.2.0` dry-run release scope.

Terminal tasks completed in this milestone:

- `TASK-43` safe release tooling and protected release-pipeline automation
- `TASK-44` publication set and package metadata hardening
- `TASK-45` package archive verification and publication-shape dependency resolution
- `TASK-46` packaged external consumer fixtures
- `TASK-47` supported compatibility matrix
- `TASK-48` compatibility policy and accepted API baseline
- `TASK-49` persisted-schema, generated-code, and CLI compatibility fixtures
- `TASK-50` changelog and contributor-guide completion
- `TASK-51` v0.2.0 release-line retargeting and status cleanup
- `TASK-52` full release-path simulation and implementation-plan alignment

Recorded outcome:

- `just release-verify v0.2.0` succeeds without publishing crates or creating production release objects.
- Negative-path release checks reject mismatched tags, missing authorization, unprotected refs, missing credentials, development-version publish attempts, and internal dependency drift.
- Retained dry-run artifacts live under `target/vvm-release/0.2.0`.
- `docs/dev/implementation-plan.md` records milestone `12.7` complete and milestone `12.8` not started.

No milestone `12.8` audit, RC publication, or final-release work has started.

---
id: m-4
title: "Milestone 12.6 — Benchmark Suite and Performance Baselines"
---

## Description

Criterion-only local benchmark suite and baseline workflow for VVM milestone 12.6. Covers pure-Rust runtime benchmarks, native Verilator-backed workloads, build and cargo-vvm subprocess benchmarks, documentation, validation, and truthful milestone closure without starting 12.7.

## Summary

Milestone 12.6 delivered a Criterion-only local benchmark suite across `vvm-core`, `vvm-build`, `cargo-vvm`, and the representative native examples. Contributor workflows now use `just benchmark`, `just benchmark-save-baseline <name>`, `just benchmark-compare-baseline <name>`, and focused `just benchmark-target` runs, with local baselines stored under `target/criterion`. The final validation path exercised the full suite, named-baseline save/compare flows, and the repository CI-equivalent checks, including restoring the workspace coverage gate to `90.06%` total line coverage after adding direct tests for `vvm-build` benchmark helpers. No benchmark CI job, custom runner, custom baseline schema, or automatic performance threshold was added, and milestone 12.7 has not been started.

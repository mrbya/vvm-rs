---
id: TASK-33
title: Benchmark packed values and randomization
status: Done
assignee: []
created_date: '2026-08-05 15:42'
updated_date: '2026-08-05 16:51'
labels:
  - benchmarking
  - criterion
  - vvm-core
milestone: m-4
dependencies:
  - TASK-32
  - TASK-35
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add Criterion benchmarks for VVM packed-value and randomization hot paths across representative widths including word boundaries. Cover extraction insertion signed conversion Bits SignedBits and replayable deterministic sequence generation using fixed seeds and black-boxed inputs and outputs.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Packed extraction is benchmarked across representative widths including 63 64 and 65 bits
- [x] #2 Packed insertion is benchmarked across representative widths including wide values
- [x] #3 Signed conversion and sign extension are benchmarked
- [x] #4 Bits and SignedBits packed operations are benchmarked
- [x] #5 Random generation is benchmarked with fixed seeds
- [x] #6 Replayable short medium and long sequence generation is benchmarked
- [x] #7 Benchmarks resist compiler elimination and use deterministic inputs
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added the dedicated `crates/vvm-core/benches/packed.rs` and `crates/vvm-core/benches/random.rs` Criterion targets, each declared with `harness = false` in `crates/vvm-core/Cargo.toml`. The packed target now benchmarks deterministic scalar extraction and insertion across representative widths `1, 8, 32, 63, 64, 65, 127, 128, 256, 1024`, including signed extraction/insertion, range-based packed extraction/insertion, equality, bit access, and `Bits` / `SignedBits` construction and sign inspection. The random target benchmarks scalar random generation, packed random generation, signed packed generation, structured transaction generation, and replayable short/medium/long sequence generation from a fixed replay token. Shared deterministic fixtures live under `crates/vvm-core/benches/packed_support/` and `crates/vvm-core/benches/random_support/`; they centralize fixed seeds, representative packed words, and replayable sequence setup. Commands executed while validating this task: `cargo check -p vvm-core --benches`, `cargo bench -p vvm-core --bench packed -- --help`, and `cargo bench -p vvm-core --bench random -- random/replay --quick`. Representative execution validation showed the replay benchmarks running successfully for 16, 256, and 4096 element sequences with stable Criterion IDs such as `random/replay/sequence/4096` and `random/replay/same-seed/4096`. Compiler-elimination resistance uses `std::hint::black_box`, and all inputs are deterministic.
<!-- SECTION:FINAL_SUMMARY:END -->

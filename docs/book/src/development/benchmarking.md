# Benchmarking

Benchmark commands live in the `justfile` and currently route through Criterion.

Useful commands:

- `just benchmark`
- `just benchmark-save-baseline NAME=local`
- `just benchmark-compare-baseline NAME=local`

Benchmarks are for performance investigation, not for proving correctness.

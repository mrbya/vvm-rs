# Common Commands

The repository command surface is intentionally centered on `just`.

## Core Validation

Use these while developing normal VVM changes:

```bash
just fmt --check
just check -- -D warnings
just test-fast
just test-native
just test-all
just doctest
just ci
```

`just ci` is the repository's CI-equivalent validation command. It does not run
benchmarks.

## Benchmark Commands

Use these for local Criterion runs:

```bash
just benchmark
just benchmark-save-baseline before-change
just benchmark-compare-baseline before-change
just benchmark-target vvm-core packed
```

Focused targets take both a package and a bench target name because VVM's suite
spans multiple workspace packages.

```bash
just benchmark-target vvm-core schedulers --save-baseline before-scheduler-change
just benchmark-target vvm-core schedulers --baseline before-scheduler-change
just benchmark-target vvm-example-counter counter
```

Benchmarks are local-only developer tools. Keep them out of `just ci`, CI jobs,
and pre-commit expectations.

# Diagnostics

VVM diagnostics are intentionally structured by phase and domain.

## What To Look For In A Failure

- build-stage context when Verilator or wrapper generation fails;
- cycle count or simulation time when a scoreboard mismatch occurs;
- replay token when a randomized test failed;
- trace and coverage output paths when artifacts were requested;
- persistence, merge, or report context for coverage-tooling failures.

## Why The Errors Are Structured This Way

VVM keeps domain-specific errors so the failure report can tell you whether the
problem happened in generation, DUT execution, comparison, tracing, timing, or
coverage persistence.

## Fast Triage Questions

1. Did the failure happen at build time or run time?
2. If it was run time, did it fail during drive, evaluation, sample, compare, or
   artifact handling?
3. If the test was randomized, do you have the replay token?

## Related Material

- [Troubleshooting](../guide/troubleshooting.md)
- [Environment Variables](environment-variables.md)

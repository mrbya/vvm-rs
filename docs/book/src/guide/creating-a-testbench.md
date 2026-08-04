# Creating A Testbench

`Testbench` is the normal cycle-driven VVM runtime builder.

## Complete Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:test}}
```

## What `Testbench` Owns

[`Testbench`](../api-guide/testbench.md) owns the execution pipeline for one DUT
run:

- the DUT instance;
- the sequence;
- the reference model;
- the scoreboard;
- the clock policy;
- optional extras such as coverage and failure policy.

## Why The Builder Order Exists

The common order is:

1. `Testbench::new(dut)`
2. `.with_sequence(...)`
3. `.with_reference_model(...)`
4. `.with_scoreboard(...)`
5. `.with_clock(...)`
6. optionally `.with_coverage(...)`
7. `.run::<Observation>()` or `.run_covered::<Observation>(...)`

That order mirrors the data flow of the verification run and keeps each required
piece explicit.

## Runtime Cycle

For the normal cycle-driven path, the runner performs these phases:

```text
drive
evaluate
clock transition
sample
predict
compare
cover
finalize
```

The exact observable order is documented in the
[Execution Order Reference](../reference/execution-order.md).

## Failure Behaviour

- a DUT access or evaluation failure becomes a simulation-stage error;
- a scoreboard mismatch becomes a retained check failure;
- coverage failures are reported in the covered path;
- finalization failures are retained in the result instead of being silently
  dropped.

## Ownership

The sequence, model, scoreboard, clock, and coverage model are moved into the
testbench because the runner owns their lifecycle for the duration of the run.

## Common Mistakes

- trying to reason about the workflow as isolated setter calls instead of a run
  lifecycle;
- mixing up observation sampling with model prediction;
- assuming a passing build means the scoreboard and model are already correct.

## Related Material

- [How VVM Works](../how-vvm-works.md)
- [Registering Tests](registering-tests.md)
- [Testbench API Guide](../api-guide/testbench.md)

# Scoreboards

A scoreboard compares the predicted result with the observed result.

## Where It Fits

The scoreboard consumes the reference model's expectation and the sampled DUT
observation.

```text
Expected observation -----> Scoreboard
                               ^
Observed DUT output ----------|
```

## Small Generalized Example

This source-backed fixture test uses [`ExactScoreboard`](../api-guide/models-and-scoreboards.md)
to turn a disagreement into a structured mismatch instead of a loose assertion.

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:scoreboard}}
```

`ExactScoreboard` is the common starting point: it checks exact equality and
returns a `Mismatch` when the observed output differs from the prediction.

## Why The Abstraction Exists

The important design point is that the scoreboard owns comparison policy. That
lets you separate:

- what traffic was generated;
- what behavior was expected;
- what counts as an error.

## What Scoreboards Own

- equality or tolerance policy;
- mismatch construction;
- any comparison-specific diagnostics.

## What Scoreboards Do Not Own

- stimulus generation;
- expected-value prediction;
- DUT execution order;
- coverage persistence.

Failures carry cycle-aware context so the final report can say which cycle or
time slot failed, not just that something failed somewhere in the run.

## Related Material

- [Creating A Testbench](../guide/creating-a-testbench.md)
- [Troubleshooting](../guide/troubleshooting.md)
- [Models And Scoreboards API Guide](../api-guide/models-and-scoreboards.md)

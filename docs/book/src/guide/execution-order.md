# Execution Order

This chapter records the observable order of major VVM runtime phases.

## Cycle-driven Order

For the normal `Testbench` path, the observable order is:

```text
drive
evaluate
inactive transition
active transition
sample
reference prediction
scoreboard comparison
coverage sampling
result accumulation
finalization
trace closure
```

In practice, one cycle-driven transaction flows like this:

1. drive the current stimulus transaction;
2. evaluate the DUT around the configured clock transitions;
3. sample the selected outputs into the observation type;
4. predict the expected observation in the reference model;
5. compare expected and observed behavior in the scoreboard;
6. sample functional coverage when attached;
7. retain diagnostics and finalize the result when the run ends.

## Single-clock Behaviour

With one primary clock, the important mental model is simple: VVM applies the
driven transaction, advances and evaluates the clocked design deterministically,
then samples and checks the result.

## Multi-clock Same-time Ordering

Multi-clock scheduling remains deterministic relative to the schedule you
configured.

- same-time ordering is explicit, not magical;
- VVM does not claim a hidden universal ordering for independent domains;
- when two domains interact, reason from the configured schedule, not from a
  guessed simulator tie-break rule.

## Failure Behaviour

The phase ordering still matters when something fails.

- drive or evaluate failures become simulation-stage errors;
- sample failures happen before reference-model prediction for that observation;
- scoreboard mismatches happen after a successful sample;
- coverage sampling happens only after the sampled cycle exists.

## Partial Results

A failed run can still retain useful state:

- cycle or time information for the failure;
- retained mismatches or simulation errors;
- replay information for randomized tests;
- coverage captured before the failure;
- traces written before shutdown.

## Timing-enabled Order

Timing-enabled execution is intentionally different.

1. the DUT reports whether delayed events are pending;
2. the timing scheduler advances to the next pending slot;
3. the DUT evaluates at that slot;
4. the scheduler records the emitted timing event;
5. the run continues until idle or until a configured bound is reached.

This is why cycle-driven and timing-driven APIs remain separate. One is built
around explicit transaction boundaries. The other is built around future work
that the HDL scheduled for itself.

## Related Material

- [Creating A Testbench](creating-a-testbench.md)
- [Multi-clock Execution](multi-clock.md)
- [Timing-enabled Models](timing-models.md)
- [How VVM Works](../how-vvm-works.md)
- [Schedulers API Guide](../api-guide/schedulers.md)

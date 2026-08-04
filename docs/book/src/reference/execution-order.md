# Execution Order

This page records the observable order of major VVM runtime phases.

## Cycle-driven Testbench Order

For the normal `Testbench` path, the observable phase order is:

1. drive the current stimulus transaction;
2. apply the configured clock transition and evaluate the DUT;
3. sample the selected outputs into the observation type;
4. predict the expected observation from the reference model;
5. compare expected and observed behaviour in the scoreboard;
6. sample functional coverage when attached;
7. retain diagnostics and finalize the result at the end of the run.

## Timing-enabled Order

Timing-enabled execution is different.

In timing mode:

1. the DUT reports whether delayed events are pending;
2. the timing scheduler advances to the next pending slot;
3. the DUT evaluates at that slot;
4. the scheduler records the emitted timing event;
5. the run continues until idle or until the configured bound is reached.

## Multi-clock Note

Multi-clock execution remains deterministic only relative to the schedule you
apply. VVM does not claim a hidden universal ordering for independent domains.

## Related Material

- [How VVM Works](../how-vvm-works.md)
- [Creating A Testbench](../guide/creating-a-testbench.md)
- [Timing-enabled Models](../guide/timing-models.md)

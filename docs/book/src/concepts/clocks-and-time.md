# Clocks And Time

VVM has two execution styles because they solve different problems.

## Where It Fits

Clocking and timing decide when the DUT is evaluated and when observations are
valid.

```text
Clock configuration
      |
      v
Drive inputs
      |
      v
Evaluate inactive phase
      |
      v
Apply active edge
      |
      v
Sample outputs
```

- Cycle-driven execution uses clocks and explicit transaction boundaries.
- Timing-enabled execution advances a Verilator model through internally
  scheduled delayed slots.

These are intentionally separate APIs.

## Small Generalized Example

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:clock}}
```

`ClockScheduler` is about deterministic ordering of externally driven clocks.
`TimingScheduler` is about future events that the HDL scheduled for itself.

## Why The Separation Exists

This distinction matters because a design can be simple in one model and awkward
in the other. Cycle-driven execution models transaction boundaries and explicit
clock ownership. Timing-enabled execution models delayed HDL activity that keeps
running between your own drives.

## What Clocks And Time Own

- deterministic event ordering;
- active and inactive phase semantics;
- advancement to the next externally driven clock edge or the next pending timed
  DUT event.

## What Clocks And Time Do Not Own

- transaction generation;
- expected-value prediction;
- mismatch policy;
- user-defined coverage meaning.

## Related Material

- [Multi-clock Execution](../guide/multi-clock.md)
- [Timing-enabled Models](../guide/timing-models.md)
- [Clocks API Guide](../api-guide/clocks.md)
- [Schedulers API Guide](../api-guide/schedulers.md)

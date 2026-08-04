# Multi-clock Execution

VVM supports deterministic multi-clock execution, but it does not hide the fact
that independent domains need an explicit schedule.

## Minimal Fixture

The generalized multi-clock fixture uses two clocks: one peripheral clock that
updates a count and one core clock that samples it.

```systemverilog
{{#include ../../../../tests/fixtures/native-multi-clock-counter/rtl/multi_clock_counter.sv}}
```

The focused test drives those domains explicitly:

```rust
{{#include ../../../../tests/fixtures/native-multi-clock-counter/src/lib.rs}}
```

## What This Teaches

- each domain still has an explicit owner;
- same-time ordering must be chosen deterministically;
- the observed behaviour is meaningful only relative to that chosen schedule.

## What VVM Guarantees

VVM guarantees deterministic execution for the schedule you implement.

It does not claim that a passing multi-clock test is a formal CDC proof or a
metastability analysis.

## Choosing Ratios And Phases

Use ratios and phase relationships that stress the interfaces you care about:

- equal rates for handshake sanity;
- non-equal rates for sampling latency;
- coincident edges when ordering matters.

## Related Material

- [Clocks And Time](../concepts/clocks-and-time.md)
- [Execution Order](execution-order.md)
- [Asynchronous FIFO case study](../examples/async-fifo.md)

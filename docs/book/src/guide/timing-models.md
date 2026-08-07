# Timing-enabled Models

Timing-enabled execution is for DUTs that schedule delayed events internally.

## Generalized Fixture

The timing fixture uses a small delayed-output design and the public
`TimingScheduler` API:

```rust
{{#include ../../../../tests/fixtures/native-timing-delay/src/lib.rs}}
```

## Mental Model

Cycle-driven tests say: "apply the next clocked step."

Timing-enabled tests say: "advance to the next pending HDL time slot." That is a
different scheduler and it stays separate on purpose.

## What You Need

- a timing-capable generated wrapper;
- a C++20 compiler with coroutine support for the Verilator timing path;
- `TimedDut` and `TimingScheduler` instead of the normal cycle-driven clock loop.

## Common Operations

- initialize the timing scheduler;
- inspect whether events are pending;
- advance to the next slot manually or run until idle;
- observe absolute simulation time and emitted timing events;
- finalize the DUT and close any open trace.

## Important Limits

- same-time and `#0` scheduling are unsupported;
- timing mode and ordinary clock scheduling remain separate;
- timing support does not make the model four-state.

## Related Material

- [Schedulers API Guide](../api-guide/schedulers.md)
- [Compatibility And Limitations](compatibility-and-limitations.md)
- [Timed UART case study](../examples/timed-uart.md)

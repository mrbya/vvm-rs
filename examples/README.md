# VVM Examples

These packages are studied in order. They are complete Rust testbenches using
only the public `vvm` and `vvm-build` APIs, not code-generation probes.

| Level | Example | Prerequisites | DUT and verification lesson |
| --- | --- | --- | --- |
| Beginner | [Counter](counter/README.md) | Rust tests and a clocked register | Typed drive/sample, one clock, reference model, exact scoreboard, replay, traces, and coverage. |
| Intermediate | [Synchronous FIFO](sync-fifo/README.md) | Counter workflow | Queue transactions, boundary behavior, a logical `VecDeque` model, and randomized ordering checks. |
| Advanced | [Timed UART](timed-uart/README.md) | Counter workflow and HDL delays | `TimingScheduler`, event timestamps, protocol reconstruction, parity, and injected errors. |
| Advanced | [Asynchronous FIFO](async-fifo/README.md) | FIFO and multiple clocks | Independently scheduled clocks and deterministic same-time edge ordering. |
| Specialist | [Tri-state bus](tri-state-bus/README.md) | Clocked testbenches | Generated inout components, caller-owned resolution, settling, and contention policy. |

Start with the counter. Run every public example with:

```bash
just test-examples
```

The focused packed-array, enum, struct, signed, wide-value, timing-delay, and
synthetic scheduling regressions live under `tests/fixtures/`; they protect
generation behavior but are intentionally not presented as learning material.
Run them with `just test-native-fixtures`.

# Schedulers

VVM exposes two scheduler families because they model different execution
domains.

- `ClockScheduler` runs externally owned clocks.
- `TimingScheduler` advances internally scheduled delayed events.

Do not treat them as interchangeable. If the DUT behavior is fundamentally about
delayed internal events, use timing mode. If the workflow is about explicit
clocked transactions, use the cycle-driven path.

Rustdoc:

- [`vvm::timing`](../api/vvm/timing/index.html)

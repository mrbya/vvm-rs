# Schedulers

VVM exposes two scheduler families because they model different execution
domains.

- `ClockScheduler` runs externally owned clocks.
- `TimingScheduler` advances internally scheduled delayed events.

Do not treat them as interchangeable. If the DUT behavior is fundamentally about
delayed internal events, use timing mode. If the workflow is about explicit
clocked transactions, use the cycle-driven path.

Rustdoc:

- [`ClockScheduler`](../api/vvm/timing/struct.ClockScheduler.html)
- [`TimingScheduler`](../api/vvm/timing/struct.TimingScheduler.html)
- [`TimingEvent`](../api/vvm/timing/struct.TimingEvent.html)
- [`TimingRun`](../api/vvm/timing/struct.TimingRun.html)
- [`SchedulerError`](../api/vvm/timing/enum.SchedulerError.html)
- [`TimingStage`](../api/vvm/timing/enum.TimingStage.html)

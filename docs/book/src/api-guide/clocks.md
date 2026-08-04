# Clocks

The `Clock` derive and clock-related timing types live under `vvm::timing`.

Use the derive when a generated DUT clock input should be driven by the normal
cycle scheduler. Use explicit timing types such as `ClockTiming`, `CycleTiming`,
`SimulationTime`, and `TimeStep` when you need to reason about execution order
or elapsed time directly.

The important conceptual split is:

- `Clock` types describe externally scheduled clocks;
- `TimingScheduler` deals with internally scheduled delayed events.

Rustdoc:

- [`Clock` derive](../api/vvm/derive.Clock.html)
- [`vvm::timing::Clock`](../api/vvm/timing/trait.Clock.html)
- [`ClockScheduler`](../api/vvm/timing/struct.ClockScheduler.html)
- [`ClockTiming`](../api/vvm/timing/struct.ClockTiming.html)
- [`CycleTiming`](../api/vvm/timing/struct.CycleTiming.html)
- [`SimulationTime`](../api/vvm/timing/struct.SimulationTime.html)
- [`TimeStep`](../api/vvm/timing/struct.TimeStep.html)

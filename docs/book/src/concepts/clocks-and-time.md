# Clocks And Time

VVM has two execution styles because they solve different problems.

- Cycle-driven execution uses clocks and explicit transaction boundaries.
- Timing-enabled execution advances a Verilator model through internally
  scheduled delayed slots.

These are intentionally separate APIs.

`ClockScheduler` is about deterministic ordering of externally driven clocks.
`TimingScheduler` is about future events that the HDL scheduled for itself.

This distinction matters because a design can be simple in one model and awkward
in the other. The counter and synchronous FIFO are natural cycle-driven examples.
The timed UART is natural in timing mode because the design emits delayed serial
events on its own.

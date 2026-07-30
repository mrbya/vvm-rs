# Timing-enabled Models

Timing mode is for Verilator models that schedule delayed events internally.

Enable it in `build.rs`, then use `TimingScheduler` on the generated timing-capable
wrapper. This is different from cycle-driven execution:

- the DUT schedules its own future slots;
- Rust advances to the next pending slot;
- observations can be tied to absolute simulation time.

The timed UART example is the canonical public guide because it reconstructs a
serial frame from delayed transitions instead of forcing the design into a fake
cycle-only structure.

Current limitations remain important:

- same-time and `#0` scheduling are unsupported;
- timing and clock schedulers stay separate on purpose.

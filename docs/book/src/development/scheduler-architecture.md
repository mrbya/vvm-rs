# Scheduler Architecture

There are two scheduler families for deliberate reasons.

- `ClockScheduler` owns deterministic externally driven clocks.
- `TimingScheduler` owns delayed future slots scheduled by the HDL itself.

They are not merged because the ownership rules and observable semantics are not
the same. A unified surface would hide important distinctions and make failures
harder to reason about.

# Timing-delay example

## Purpose

This example verifies a finite SystemVerilog `initial` process containing
positive delays. The model declares `timeunit 1ns` and `timeprecision 1ns`, so
each reported simulator tick is one nanosecond.

## Build-time timing support

```rust
DutBuilder::new("delayed_sequence")
    .top_module("delayed_sequence")
    .source("rtl/delayed_sequence.sv")
    .timing()
    .build()
```

`.timing()` enables Verilator timing constructs, generates a `TimedDut`, links
the Verilator timing runtime, and uses a coroutine-capable C++ configuration.
It does not integrate timing events into the ordinary `Testbench`.

## Manual stepping

```rust
let mut scheduler = TimingScheduler::new();

scheduler.initialize(&mut dut)?;

while let Some(event) = scheduler.advance_next(&mut dut)? {
    println!("slot {} at {}", event.ordinal(), event.time());
}
```

## Bounded execution

```rust
let run = scheduler.run_until_idle(&mut dut, max_slots)?;
```

This process reports `time_slots = 3` and `evaluations = 4`, because scheduler
initialization performs one evaluation.

## Timeline

```text
Time | Value | Done  | Meaning
-----|-------|-------|---------------------------
0    | 0x00  | false | initial evaluation
2    | 0x11  | false | first delayed slot
5    | 0x22  | false | second delayed slot
10   | 0x33  | true  | final delayed slot
```

## Finalization

`TimingScheduler` does not call `finish()`. Call it explicitly after execution
when the DUT lifecycle and any active trace must be completed.

## Tracing

Open tracing before initialization to capture time zero. Evaluations create
waveform dumps; explicit `finish()` flushes and closes the trace.

## Cycle mode versus timing mode

```text
Cycle mode:
    Rust schedules external input-clock transitions.

Timing mode:
    Verilator reports internal delayed-event times.

The schedulers are separate.
```

## Limitations

- Positive future time slots only.
- No `#0`; same-time slots are rejected.
- Execution must be finite or explicitly bounded.
- No automatic finalization.
- No automatic `Testbench` integration.
- No hybrid clock-plus-delay loop yet.
- One DUT per scheduler invocation.

## Running

```console
cargo test --manifest-path tests/fixtures/native-timing-delay/Cargo.toml
```

```console
cargo test --manifest-path tests/fixtures/native-timing-delay/Cargo.toml
```

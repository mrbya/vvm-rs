# Timed UART

## Purpose

This advanced example verifies a compact behavioral UART transmitter through
Verilator timing support. It is a protocol-verification demonstration, not a
general UART library or synthesizable UART implementation.

## Design overview

```text
data + parity controls --> [ behavioral timed transmitter ] --> tx
                                      |                       busy, done
                                      +-- #10 ns per bit
```

The model uses `` `timescale 1ns/1ps`` and schedules each bit after a positive
10 ns delay. Frames are idle-high, one low start bit, eight LSB-first data bits,
an optional even/odd parity bit, one stop bit, then final idle. Error inputs
invert parity or force the stop bit low.

## Verification goals

Tests configure inputs before scheduler initialization, use `TimingScheduler`
to walk every event slot, record typed `(time, tx)` observations, reconstruct a
frame, and compare its data, parity, and stop bit to an independent protocol
model. They also explicitly call `finish`.

## VVM features demonstrated

`DutBuilder::timing`, `TimedDut`, `TimingScheduler`, trace opening, absolute
event times, protocol reconstruction, explicit finalization, and manually
captured functional coverage.

## Run commands

```bash
cargo nextest run -p vvm-example-timed-uart
cargo test -p vvm-example-timed-uart uart_detects_injected_parity_error
cargo vvm coverage --output target/timed-uart-coverage --name timed-uart -- \
    test -p vvm-example-timed-uart uart_random_frames
```

All eight protocol tests pass. The known-byte test covers zero, other tests
cover maximum and alternating patterns, parity modes, and both error injections.
`uart_random_frames` captures parity mode, requested versus observed error state,
and data-class coverage; `cargo vvm coverage` persists per-test artifacts and
merged text and HTML reports. `just functional-coverage-timed-uart` runs the
same workflow.

## Known limitations

This model sends one frame during its `initial` process; it has no baud-rate
generator, receiver, handshake, or back-to-back queue. It demonstrates the
current timing API only; same-time and `#0` scheduling remain unsupported.

## Suggested next example

Continue with the asynchronous FIFO for independently scheduled clocks.

# Asynchronous FIFO

## Purpose

This advanced example replaces the synthetic public multi-clock counter. It
uses independent read and write clocks around a compact dual-domain FIFO.

## Design overview

```text
write clock --> [ binary/Gray write pointer ] --> two-flop crossing --> [ read domain ] --> read clock
                     |                                                  ^
                     +---------------- eight-entry memory -------------+
```

The RTL uses binary pointers locally, Gray-coded pointer crossings, and two-stage
synchronizers. The directed tests exercise write-faster, read-faster,
wraparound, empty and full rejection, and a deterministic write-before-read
coincident-edge policy.

## Verification goals and VVM features

The example demonstrates two generated clock inputs, independent resets,
explicit deterministic edge ordering, a logical `VecDeque` reference model,
and public generated-DUT accessors. The test code advances both domains
explicitly so each synchronization latency remains visible to readers.

```bash
cargo nextest run -p vvm-example-async-fifo
```

All directed scenarios pass, including deterministic randomized traffic seeded
by `chacha8-v1:a579c102`. `async_fifo_random` accepts the standard
`VVM_REPLAY` override and reports its replay token through the manual-result
adapter used by this direct multi-domain example. Its manual coverage group
records accepted and rejected read/write operations, written-data classes, and
empty, available, and full model occupancy. Persist and report those samples
with:

```bash
cargo vvm coverage --output target/async-fifo-coverage --name async-fifo -- test -p vvm-example-async-fifo async_fifo_random
```

## Known limitations

This example verifies functional behavior under deterministic clock schedules.
It is not a formal CDC or metastability proof. It does not model metastability,
physical timing closure, or arbitrary reset deassertion. The focused synthetic
scheduler regression remains under `tests/fixtures/native-multi-clock-counter`.

## Suggested next example

Continue with the tri-state bus for caller-owned inout resolution.

# Synchronous FIFO

## Purpose

This is the primary sequential-design example after the counter. It verifies an
eight-entry, one-clock FIFO with typed push/pop transactions and a queue-based
Rust model.

## Design overview

```text
push, push_data --> [ 8 x 8 FIFO ] --> pop_data
pop              --> [ pointers   ] --> empty, full, occupancy
```

`reset_n` is active low. A push is accepted unless the FIFO was full at the
edge; a pop is accepted unless it was empty. At full, simultaneous push/pop
rejects the push and accepts the pop. At empty, it accepts the push and rejects
the pop. `overflow` and `underflow` are one-cycle indicators. `pop_data` holds
the most recently accepted pop value.

## Verification goals

- Preserve data order through fill, drain, and pointer wraparound.
- Check occupancy and flags on every edge with `VecDeque<u8>`, not RTL pointers.
- Exercise underflow, overflow, simultaneous operations, VCD tracing, and a
  replayable randomized regression.

## VVM features demonstrated

`DutBuilder`, `include_dut!`, derived `Drive` and `Sample`, `Clock`,
`Testbench`, `ReferenceModel`, `ExactScoreboard`, `VVM_REPLAY`, `VVM_CYCLES`,
and trace-capable `#[vvm::test]`.

## Project layout

```text
sync-fifo/
├── build.rs
├── rtl/sync_fifo.sv
└── src/lib.rs
```

## Run commands

```bash
cargo nextest run -p vvm-example-sync-fifo
VVM_REPLAY=chacha8-v1:0000000000000000000000004649464f cargo test -p vvm-example-sync-fifo fifo_random
VVM_TRACE_DIR=target/fifo-traces cargo test -p vvm-example-sync-fifo fifo_smoke
```

All five tests pass and the smoke test writes a VCD when tracing is requested.
The randomized test prints a replay token on failure. `FifoCoverage` records
idle, push, pop, simultaneous operations, empty/available/full boundaries, and
their two-way cross. Run the persisted-artifact workflow with:

```bash
cargo vvm coverage --output target/fifo-coverage --name sync-fifo -- test -p vvm-example-sync-fifo fifo_
```

## Known limitations

The DUT is a fixed 8-bit, depth-8 synchronous FIFO. It is not an interface,
ready/valid, or CDC example. Continue with the timed UART or asynchronous FIFO.

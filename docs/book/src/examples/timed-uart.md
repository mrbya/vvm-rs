# Timed UART

The timed UART is the main timing-enabled public example.

It introduces:

- `DutBuilder::timing()`;
- `TimingScheduler`;
- absolute event times;
- protocol reconstruction from delayed output transitions;
- explicit `finish()` handling;
- manual coverage capture around reconstructed frames.

File to study first: `examples/timed-uart/src/lib.rs`.

Run it with:

```bash
cargo nextest run -p vvm-example-timed-uart
```

Read this after you understand the cycle-driven counter and FIFO workflows.

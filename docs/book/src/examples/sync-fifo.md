# Synchronous FIFO

The synchronous FIFO is the first example where the verification logic is more
interesting than the raw generated wrapper.

It introduces:

- semantic push/pop transactions;
- a manual `Sample` implementation that presents a clearer observation contract;
- a `VecDeque` reference model;
- boundary behavior such as empty, available, and full;
- randomized traffic plus typed coverage.

File to study first: `examples/sync-fifo/src/lib.rs`.

Run it with:

```bash
cargo nextest run -p vvm-example-sync-fifo
```

Read this after the counter when you want a realistic single-clock queue example.

# Asynchronous FIFO

The asynchronous FIFO is the main multi-clock public example.

It introduces:

- independent read and write edges;
- deterministic same-time ordering;
- synchronizer latency that remains visible in the test code;
- direct model-driven checking without hiding clock ownership;
- manual coverage for accepted and rejected transfers.

Files to study:

- `examples/async-fifo/src/verification.rs`
- `examples/async-fifo/src/test_cases.rs`

Run it with:

```bash
cargo nextest run -p vvm-example-async-fifo
```

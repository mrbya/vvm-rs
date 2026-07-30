# Tri-state Bus

The tri-state bus is the specialist end of the example ladder.

It introduces:

- generated split inout APIs;
- caller-owned resolution policy;
- explicit floating-bit policy;
- deterministic contention detection;
- bounded settling at one simulation time.

Files to study:

- `examples/tri-state-bus/src/resolution.rs`
- `examples/tri-state-bus/src/verification.rs`
- `examples/tri-state-bus/src/lib.rs`

Run it with:

```bash
cargo nextest run -p vvm-example-tri-state-bus
```

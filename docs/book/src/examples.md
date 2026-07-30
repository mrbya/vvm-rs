# Examples Overview

The curated example ladder is part of the public documentation, not a side list.

Recommended order:

1. [Counter](examples/counter.md)
2. [Synchronous FIFO](examples/sync-fifo.md)
3. [Timed UART](examples/timed-uart.md)
4. [Asynchronous FIFO](examples/async-fifo.md)
5. [Tri-state Bus](examples/tri-state-bus.md)

Run the whole ladder with:

```bash
just test-examples
```

The focused generated-port regressions under `tests/fixtures/` are important for
correctness but are not presented as the learning path.

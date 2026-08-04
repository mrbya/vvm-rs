# Examples Overview

The curated examples are case studies.

Use the Guide and Quick Start to learn the generalized workflow first. Then use
these examples to see how the same patterns scale to more realistic designs.

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

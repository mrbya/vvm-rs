# Multi-clock Execution

VVM supports deterministic multi-clock execution, but it does not hide the fact
that independent clocks need an ordering policy.

The asynchronous FIFO example is the main public workflow for this area. It uses
explicit helper functions to advance write and read edges separately so readers
can see the synchronization latency directly.

The key points are:

- each clock domain still has an explicit owner;
- same-time ordering is deterministic, not arbitrary;
- the example demonstrates functional behavior under a chosen schedule, not a
  formal CDC proof.

Use the async FIFO example when you need a real multi-clock pattern to copy.

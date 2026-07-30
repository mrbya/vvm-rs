# Concepts Overview

VVM is easiest to use when you understand the workflow before you memorize the
APIs. The core idea is simple: drive typed transactions into a generated DUT,
sample typed observations back out, compare those observations to a reference
model, and retain useful diagnostics when they differ.

The following chapters explain the roles in that workflow:

- the generated DUT wrapper and what it owns;
- transactions and sequences;
- reference models and scoreboards;
- clocks, time, and deterministic ordering;
- failures, reports, and replay.

If you only need the first working path, use [Quick Start](../quick-start.md).
If you already know the concepts and need concrete APIs, jump to the
[API Guide](../api-guide/overview.md).

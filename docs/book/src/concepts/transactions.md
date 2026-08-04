# Transactions

A transaction is the typed unit of meaning you want to apply or observe.

## Where It Fits

Transactions sit at the edge of the verification pipeline, between sequence
generation and DUT driving or output sampling.

```text
Sequence -> transaction -> Drive/Sample -> DUT boundary
```

## Small Generalized Example

This fixture-backed type is a transaction because it expresses the inputs that a
test cares about, not every RTL implementation detail.

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:stimulus}}
```

In a counter, a transaction might be just `reset_n` and an event pulse. In a
FIFO, it is usually a higher-level push/pop request instead of raw internal
flags. In a bus-oriented interface, it may need separate request and response
types.

Good VVM transaction types usually follow these rules:

- They describe behavior at the boundary you care about.
- They hide irrelevant DUT internals.
- They are cheap to clone or copy when practical.
- They derive `Drive` or `Sample` only for the fields that map to ports.

## Why The Abstraction Exists

The goal is not to mirror the RTL implementation line by line. The goal is to
create types that keep the verification flow readable and let the reference
model, scoreboard, and coverage model work in terms of meaningful operations.

## What Transactions Own

- the externally meaningful data for one operation or observation;
- naming that matches the verification intent;
- optional explicit port mappings for the fields that cross the DUT boundary.

## What Transactions Do Not Own

- sequence generation policy;
- clocking or timing order;
- expected-result prediction;
- pass/fail comparison logic.

## Related Material

- [Driving Inputs](../guide/driving-inputs.md)
- [Sampling Outputs](../guide/sampling-outputs.md)
- [Drive And Sample API Guide](../api-guide/drive-and-sample.md)

# Randomization And Replay

Deterministic pseudo-random testing is one of VVM's most practical workflows:
you get variability without giving up reproducibility.

## Generalized Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:replayable-sequence}}
```

And the registered test that consumes it:

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:replay-test}}
```

## Mental Model

The sequence owns the mapping from random bits to stimulus. That mapping is part
of the reproducibility contract. If you change it, the same replay token can stop
describing the same stimulus stream.

## Precedence

For replay-capable tests, the important precedence is:

1. `VVM_REPLAY`
2. `VVM_SEED`
3. the default replay token declared by the test

Replay is the strongest reproduction control because it reconstructs the exact
pseudo-random stream the sequence expects.

## Reproducing A Failure

```bash
VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test event_counter_random
```

## Useful Design Rules

- keep the random-to-stimulus mapping stable;
- force a deterministic reset preamble when the DUT needs one;
- avoid implying a constraint solver that VVM does not provide;
- record the replay token whenever a randomized test fails in CI.

## What Replay Does Not Guarantee

Replay reconstructs the pseudo-random sequence. It does not magically fix a test
that also depends on uncontrolled external state.

## Related Material

- [Configuring Tests](configuring-tests.md)
- [Randomization API Guide](../api-guide/randomization.md)
- [Counter case study](../examples/counter.md)

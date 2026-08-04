# Sequences

VVM deliberately uses ordinary Rust iterators for stimulus sequences. That means
you can write:

## Where It Fits

Sequences choose the next transaction. They do not drive the DUT or check the
result.

```text
Sequence
   |
   v
Stimulus transaction
   |
   v
Drive
```

- fixed directed sequences;
- pseudo-random replayable sequences;
- sequences built from combinators;
- small helper generators in normal Rust modules.

## Small Generalized Example

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:sequence}}
```

This design keeps VVM aligned with the language and avoids inventing a second
mini-language for sequence generation.

## Why The Abstraction Exists

The main practical rule is to keep the iterator contract clear. A sequence is
responsible for choosing transactions, not for evaluating the DUT or checking
results. That separation is what makes replay and debugging manageable.

## What Sequences Own

- transaction ordering;
- deterministic directed traffic or randomized traffic generation;
- replay identity when the sequence is replayable.

## What Sequences Do Not Own

- DUT lifecycle;
- sampled observations;
- expected-value prediction;
- scoreboard comparison policy.

## Related Material

- [Registering Tests](../guide/registering-tests.md)
- [Randomization And Replay](../guide/randomization-and-replay.md)
- [Randomization API Guide](../api-guide/randomization.md)

# Randomization And Replay

Replayable randomization is one of VVM's most practical features: randomized
traffic stays reproducible.

The normal pattern is:

- define a stable default replay token in the test or example;
- construct a `RandomContext` or replayable sequence from that token;
- let `VVM_REPLAY` override the default when reproducing failures.

Precedence for replay-capable tests is:

1. `VVM_REPLAY`
2. `VVM_SEED`
3. the default replay token declared by the test
4. generated randomness when the API allows it

The sequence itself should own how random words map to stimulus. That mapping is
part of the reproducibility contract and should not change casually.

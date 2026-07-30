# Randomization

The randomization surface lives under `vvm::random`.

Important types:

- `Seed`
- `ReplayToken`
- `RandomContext`
- `ReplayableSequence`
- `Randomize`

The most common user pattern is to store a stable default `ReplayToken`, then
let `TestRunConfig` replace it with a runtime override.

Rustdoc:

- [`vvm::random`](../api/vvm/random/index.html)

# Counter Example

Standard Rust test-harness integration for a generated VVM DUT.

- `reset_n` is active low.
- `count` resets to zero when `reset_n` is low.
- `count` increments by one on each rising clock edge when `enable` is high and reset is deasserted.
- `cargo test -p vvm-example-counter counter_smoke` runs the smoke test through libtest.
- `cargo test -p vvm-example-counter counter_random` runs the randomized regression.
- `cargo test -p vvm-example-counter counter_fail -- --ignored` runs the intentionally failing reporting example.

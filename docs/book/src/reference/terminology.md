# Terminology

- DUT: generated device wrapper used by test code.
- Stimulus: typed input transaction driven into the DUT.
- Observation: typed sampled output state.
- Sequence: ordinary iterator that emits stimulus transactions.
- Reference model: expected-behavior model in ordinary Rust.
- Scoreboard: comparison policy between expected and observed results.
- Observed cycle: one sampled cycle with both stimulus and observation context.
- Timing slot: one internally scheduled delayed-event step in timing mode.
- Coverage artifact: immutable persisted result of one captured coverage session.

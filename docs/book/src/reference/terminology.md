# Terminology

- DUT: the generated wrapper around the elaborated HDL model used by test code.
- Generated DUT wrapper: the Rust module and types produced from `build.rs` and
  included with `vvm::include_dut!`.
- Stimulus transaction: one typed semantic input step driven into the DUT.
- Observation: one typed sampled output state captured after evaluation.
- Sequence: the ordered stream of stimulus transactions, often implemented as an
  iterator.
- Reference model: the behavioural model that predicts the expected observation.
- Scoreboard: the comparison policy between expected and observed behaviour.
- Clock: the policy object that applies clock transitions for cycle-driven tests.
- Simulation time: the DUT-visible time value tracked by the wrapper and timing
  scheduler.
- Observed cycle: one recorded cycle carrying both the stimulus and sampled
  observation.
- Timing slot: one internally scheduled delayed-event step in timing mode.
- Coverage model: the Rust-side model that records whether meaningful behaviours
  were exercised.
- Coverage artifact: one persisted result of a captured coverage session.
- Replay token: the deterministic identifier that reconstructs a pseudo-random
  sequence.

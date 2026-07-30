# Sampling

Coverage sampling happens at the observation point, after the test has enough
context to describe what just happened.

For cycle-driven tests that usually means after a successful sampled cycle. For
manual timing or multi-clock workflows it may mean an explicit call from the test
body once the relevant protocol or transfer has been reconstructed.

`ObservedCycle` exists so typed coverage logic can inspect both the stimulus and
the resulting observation together.

Sampling can still produce partial results and diagnostics. That is useful when
the run failed after reaching some, but not all, intended behavior.

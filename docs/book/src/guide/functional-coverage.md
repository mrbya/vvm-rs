# Functional Coverage

Functional coverage answers: "Did this test suite exercise the meaningful
behaviors I care about?"

It is different from Rust source coverage. Rust source coverage tells you which
Rust lines executed. VVM functional coverage tells you which design behaviors,
interface states, and combinations were observed.

## First End-to-end Model

Start with one small typed model that describes semantic behavior, not raw port
toggle counts.

```text
Stimulus or observation
        |
        v
Typed coverage model
        |
        v
Sampled coverpoints
        |
        v
Cross coverage
        |
        v
Per-test artifact
```

This source-backed fixture model records semantic activity and sampled output
regions for a small event-driven interface.

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:coverage-model}}
```

The important point is the separation of roles:

- the testbench still drives, samples, predicts, and compares;
- the coverage model only classifies what was observed;
- the persisted artifact is written after the run, not by mutating shared global
  state during unrelated tests.

## Bins

Bins are the named buckets that turn raw values into verification intent.

- Normal bins contribute toward coverage goals.
- Ignore bins classify values that should not count against the goal.
- Illegal bins call out values that should not happen at all.

The fixture model above uses all three forms:

- `idle` and `event` are normal bins.
- `reset` is an ignore bin because reset cycles are expected but usually should
  not dilute the goal for normal operation.
- `invalid` is an illegal bin for a reserved semantic state.

Use bins to ask meaningful questions. "How often did the interface idle versus
transfer?" is usually better than "How often did bit 0 equal 1?"

## Coverpoints

A coverpoint maps one sampled value into named bins. In VVM, that value can come
from the sampled observation, the active stimulus, or a value derived from both.

That is why `ObservedCycle` matters: it gives the sampling function access to the
transaction that was active, the sampled observation, and the cycle/time context
for that sample.

Typical coverpoints answer one-dimensional questions such as:

- which operation occurred;
- which region the output value fell into;
- whether a response was successful, retried, or rejected.

## Typed Coverage Models

`#[derive(vvm::Coverage)]` groups related coverpoints and crosses into one typed
definition with a stable definition name, revision, and instance path.

Use a typed model when the structure belongs to a design or reusable subsystem.
Use manual coverage construction when the sampling logic is ad hoc or assembled
dynamically.

The derive is a good fit when you want all of these at once:

- a stable definition fingerprint for persistence and merging;
- explicit sample functions for each measured dimension;
- a single model instance attached to a testbench run.

## Sampling And `ObservedCycle`

Sampling happens after a successful observation has been produced for a cycle.
The sample function sees an [`ObservedCycle`](../api-guide/testbench.md) so it
can derive semantic values from both sides of the interface.

That keeps coverage aligned with verification intent. For example, you can sample
"accepted write", "idle", or "backpressured read" instead of sampling only raw
handshake bits.

## Cross Coverage

Crosses answer combination questions.

In the fixture model, `activity_x_total` records which output region appeared for
each semantic activity. That is often more valuable than knowing only that each
axis was seen independently.

Keep crosses selective. If a cross does not answer a question that affects your
confidence in the design, it usually adds noise rather than insight.

## Attaching Coverage To A Testbench

Attach the model to the testbench, then run the covered path with a
`TestContext` so the harness can capture a per-test coverage snapshot.

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:coverage-test}}
```

This is the normal covered workflow:

1. Build the DUT.
2. Configure tracing or other runtime options if needed.
3. Construct `Testbench` with sequence, reference model, scoreboard, and clock.
4. Add `.with_coverage(...)`.
5. Run `run_covered::<Observation>(context)`.

## Per-test Artifacts And Snapshots

Each covered run produces one immutable coverage snapshot. That snapshot can be
retained in the test result and persisted as a per-test artifact.

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:coverage-artifact}}
```

The important artifact ideas are:

- coverage is test-scoped;
- each artifact retains provenance about where it came from;
- VVM does not fabricate coverage for tests that captured nothing;
- later merge and report steps operate on persisted artifacts, not on hidden
  shared mutable state.

## Definition Fingerprints And Provenance

Every typed or manual coverage definition has a structural fingerprint. That
fingerprint keeps unrelated definitions from being merged by accident.

Provenance records where a captured artifact came from, which matters when you
need to explain why one report changed or why one failing test contributed a
particular bin hit.

## Merge Compatibility

Coverage merging is intentionally offline.

- Compatible fingerprints can be merged.
- Incompatible definitions are rejected instead of silently combined.
- Merge policy decides which test outcomes contribute to the final aggregate.

This is why coverage guidance belongs before `cargo-vvm`: you should understand
what artifacts mean before a repository-level command starts collecting and
merging them for you.

## Reports

After merging, VVM can render:

- merged JSON for machine consumption;
- text for CI logs and quick inspection;
- HTML for browser-based review;
- GitLab metric output for coverage extraction.

The reporting goal is not only one percentage. It is an explanation of which
bins, coverpoints, and crosses were hit, missed, ignored, or classified as
illegal.

## Failure Behaviour

Coverage is related to pass/fail status, but it is not the same thing.

- A test can fail and still leave behind useful coverage.
- Illegal bins can surface a coverage-specific failure condition.
- Reporting or persistence can fail separately from the scoreboard or DUT run.

Treat coverage artifacts as additional diagnostic evidence, not as a replacement
for a correct scoreboard.

## CI Integration

The common CI flow is:

1. Run tests that emit per-test coverage artifacts.
2. Merge compatible artifacts offline.
3. Publish merged JSON, text, and HTML reports.
4. Expose the GitLab metric line if your pipeline wants a single extracted
   percentage.

For the repository-level command that runs the child test command, gathers
artifacts, and writes reports in one step, continue to
[Using cargo-vvm](using-cargo-vvm.md).

## Common Mistakes

- measuring raw implementation details instead of semantic behavior;
- counting reset or invalid traffic as normal goal progress;
- building very large crosses with no clear verification question;
- treating coverage as proof of correctness instead of evidence of exercised
  scenarios;
- trying to merge artifacts after changing the coverage definition without
  updating the revision or understanding fingerprint compatibility.

## Related Material

- [Creating A Testbench](creating-a-testbench.md)
- [Using cargo-vvm](using-cargo-vvm.md)
- [Coverage API Guide](../api-guide/coverage.md)
- [Reports API Guide](../api-guide/reports.md)
- [Coverage JSON Schema v1](../development/coverage-json-schema-v1.md)

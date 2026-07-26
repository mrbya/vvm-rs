# Ergonomic Functional Coverage

Derive a typed model when coverage follows observed transactions. Keep bins and
semantic extraction in ordinary Rust functions; VVM generates only ownership,
visitation, sampling, and cross routing.

```rust
#[derive(vvm::Coverage)]
#[vvm(definition = "decoder", revision = 1, stimulus = Stimulus, observation = Observation)]
struct DecoderCoverage {
    #[vvm(coverpoint(build = opcode_coverpoint, sample = opcode_for))]
    opcode: vvm::Coverpoint<Opcode>,
}
```

Each coverpoint builder takes its field name and returns `Result<Coverpoint<T>,
CoverageBuildError>`. Extractors receive `ObservedCycle<'_, Stimulus,
Observation>`. A cross uses `#[vvm(cross(left = opcode, right = response))]`;
the derive routes the exact returned coverpoint samples.

`DecoderCoverage::new("dut.decoder")` returns a validated `CoverageInstance`.
Attach it with `.with_coverage(...)` and use `.run_covered(context)`. Sampling
occurs at the same point as `run_with_observer`: after successful observation
and before scoreboard checking. The first coverage error stops coverage mutation
but not simulation, and the partial group is captured. Errors become deferred
framework diagnostics without replacing the testbench result.

Declare this contract with `#[vvm::test(coverage)]`. Such tests require `&mut
TestContext`; a completed testbench run that captures no group receives a
missing-coverage diagnostic. Manual `CoverageGroup`, `capture_coverage`, and
`run_with_observer` remain available for advanced workflows. Persistence,
offline merging, and reporting remain explicit per-test/offline operations.
# Suite workflow

Inside each test use derive, `with_coverage`, and `run_covered`. Outside the
suite, run `cargo vvm coverage`; see [coverage orchestration](coverage-orchestration.md).

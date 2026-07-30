# Counter

The counter is the beginner example and the best first reference for real VVM
code.

It introduces:

- `DutBuilder` and `include_dut!`
- `Drive`, `Sample`, and `Clock`
- a fixed directed sequence
- a replayable random sequence
- a stateful reference model
- `ExactScoreboard`
- trace-capable and coverage-capable tests

Files to study:

- `examples/counter/build.rs`
- `examples/counter/src/verification.rs`
- `examples/counter/src/test_cases.rs`
- `examples/counter/src/coverage.rs`

Run it with:

```bash
cargo test -p vvm-example-counter counter_smoke
```

Read this example before every other curated example.

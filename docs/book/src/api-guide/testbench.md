# Testbench

`Testbench` is the main cycle-driven orchestration API.

Typical builder stages:

1. create the testbench with the DUT;
2. attach a sequence;
3. attach a reference model;
4. attach a scoreboard;
5. attach a clock;
6. optionally attach coverage;
7. run and inspect the returned `TestResult`.

The testbench owns the lifecycle ordering. That is why the workflow chapters talk
about drive, evaluate, sample, predict, compare, and record as separate phases.

Use the Guide chapter first if you need the workflow itself. Use this page when
you need the exact surface names such as `with_sequence`, `with_replayable_sequence`,
`with_scoreboard`, `with_clock`, `with_coverage`, `run`, and `run_covered`.

Rustdoc:

- [`Testbench`](../api/vvm/testbench/struct.Testbench.html)
- [`Testbench::with_sequence`](../api/vvm/testbench/struct.Testbench.html#method.with_sequence)
- [`Testbench::with_replayable_sequence`](../api/vvm/testbench/struct.Testbench.html#method.with_replayable_sequence)
- [`Testbench::with_scoreboard`](../api/vvm/testbench/struct.Testbench.html#method.with_scoreboard)
- [`Testbench::with_clock`](../api/vvm/testbench/struct.Testbench.html#method.with_clock)
- [`Testbench::with_coverage`](../api/vvm/testbench/struct.Testbench.html#method.with_coverage)
- [`Testbench::run`](../api/vvm/testbench/struct.Testbench.html#method.run)
- [`Testbench::run_covered`](../api/vvm/testbench/struct.Testbench.html#method.run_covered)
- [`ObservedCycle`](../api/vvm/testbench/struct.ObservedCycle.html)
- [`TestResult`](../api/vvm/testbench/struct.TestResult.html)

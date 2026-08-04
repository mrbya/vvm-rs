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

- [`vvm::testbench`](../api/vvm/testbench/index.html)

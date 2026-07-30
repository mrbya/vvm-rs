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

Rustdoc:

- [`vvm::testbench`](../api/vvm/testbench/index.html)

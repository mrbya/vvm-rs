# Errors

VVM keeps errors close to the subsystem that produced them.

Common user-facing error families include:

- `BuildError` and `BuildStage` from `vvm-build`;
- generated DUT errors from the included wrapper;
- simulation and scoreboard errors from `vvm::testbench`;
- scheduler and time errors from `vvm::timing`;
- coverage build, sample, merge, session, and persistence errors from
  `vvm::coverage`.

The important practical point is that stage and source-chain information are
retained. Users should not need to guess whether a failure came from Verilator,
generated code, a runtime mismatch, or coverage reporting.

Rustdoc:

- [`vvm_build`](../api/vvm_build/index.html)
- [`vvm::testbench`](../api/vvm/testbench/index.html)
- [`vvm::timing`](../api/vvm/timing/index.html)
- [`vvm::coverage`](../api/vvm/coverage/index.html)

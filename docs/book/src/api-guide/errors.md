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

- [`BuildError`](../api/vvm_build/enum.BuildError.html)
- [`BuildStage`](../api/vvm_build/enum.BuildStage.html)
- [`SimulationError`](../api/vvm/testbench/struct.SimulationError.html)
- [`SimulationStage`](../api/vvm/testbench/enum.SimulationStage.html)
- [`SchedulerError`](../api/vvm/timing/enum.SchedulerError.html)
- [`ClockConfigurationError`](../api/vvm/timing/enum.ClockConfigurationError.html)
- [`vvm::coverage::BuildError`](../api/vvm/coverage/enum.BuildError.html)
- [`vvm::coverage::RuntimeError`](../api/vvm/coverage/enum.RuntimeError.html)
- [`vvm::coverage::SampleError`](../api/vvm/coverage/enum.SampleError.html)
- [`PersistenceError`](../api/vvm/coverage/artifact/enum.PersistenceError.html)
- [`MergeError`](../api/vvm/coverage/merge/enum.MergeError.html)
- [`SessionError`](../api/vvm/coverage/session/enum.SessionError.html)

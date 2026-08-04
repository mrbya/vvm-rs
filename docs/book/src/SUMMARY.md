# Summary

- [Introduction](introduction.md)
- [Why VVM?](why-vvm.md)
- [How VVM Fits Into HDL Verification](how-vvm-fits-into-hdl-verification.md)
- [How VVM Works](how-vvm-works.md)
- [Rust Essentials For HDL Engineers](rust-essentials-for-hdl-engineers.md)
- [Installation](installation.md)
- [Quick Start](quick-start.md)
- [Getting Started Overview](getting-started.md)

# Concepts

- [Overview](concepts/overview.md)
- [Verification Workflow](concepts/verification-workflow.md)
- [DUT And Generated Wrapper](concepts/dut-and-wrapper.md)
- [Transactions](concepts/transactions.md)
- [Sequences](concepts/sequences.md)
- [Reference Models](concepts/reference-models.md)
- [Scoreboards](concepts/scoreboards.md)
- [Clocks And Time](concepts/clocks-and-time.md)
- [Failures And Results](concepts/failures-and-results.md)

# Guide

- [User Guide Overview](user-guide.md)
- [Project Setup](guide/project-setup.md)
- [Build Script](guide/build-script.md)
- [Including The DUT](guide/including-the-dut.md)
- [Driving Inputs](guide/driving-inputs.md)
- [Sampling Outputs](guide/sampling-outputs.md)
- [Creating A Testbench](guide/creating-a-testbench.md)
- [Registering Tests](guide/registering-tests.md)
- [Configuring Tests](guide/configuring-tests.md)
- [Randomization And Replay](guide/randomization-and-replay.md)
- [Waveform Tracing](guide/waveform-tracing.md)
- [Multi-clock Execution](guide/multi-clock.md)
- [Timing-enabled Models](guide/timing-models.md)
- [Bidirectional Ports](guide/bidirectional-ports.md)
- [Troubleshooting](guide/troubleshooting.md)

# API Guide

- [Overview](api-guide/overview.md)
- [Facade And Prelude](api-guide/facade-and-prelude.md)
- [DutBuilder](api-guide/dut-builder.md)
- [Drive And Sample](api-guide/drive-and-sample.md)
- [Clocks](api-guide/clocks.md)
- [Testbench](api-guide/testbench.md)
- [Models And Scoreboards](api-guide/models-and-scoreboards.md)
- [Test Attribute](api-guide/test-attribute.md)
- [Configuration And Context](api-guide/configuration-and-context.md)
- [Randomization](api-guide/randomization.md)
- [Tracing](api-guide/tracing.md)
- [Schedulers](api-guide/schedulers.md)
- [Inout](api-guide/inout.md)
- [Coverage](api-guide/coverage.md)
- [Reports](api-guide/reports.md)
- [Errors](api-guide/errors.md)

# Functional Coverage

- [Overview](coverage.md)
- [Bins](coverage/bins.md)
- [Coverpoints](coverage/coverpoints.md)
- [Typed Models](coverage/typed-models.md)
- [Crosses](coverage/crosses.md)
- [Sampling](coverage/sampling.md)
- [Sessions And Artifacts](coverage/sessions-and-artifacts.md)
- [Schema](coverage/schema.md)
- [Merging](coverage/merging.md)
- [Reporting](coverage/reporting.md)
- [CI](coverage/ci.md)

# cargo-vvm

- [Overview](cargo-vvm.md)
- [Installation](cargo-vvm/installation.md)
- [Workflow](cargo-vvm/workflow.md)
- [Command Reference](cargo-vvm/command-reference.md)
- [Output Layout](cargo-vvm/output-layout.md)
- [Merge Policies](cargo-vvm/merge-policies.md)
- [Failure Semantics](cargo-vvm/failure-semantics.md)
- [GitLab CI](cargo-vvm/gitlab.md)
- [Troubleshooting](cargo-vvm/troubleshooting.md)

# Examples

- [Overview](examples.md)
- [Counter](examples/counter.md)
- [Synchronous FIFO](examples/sync-fifo.md)
- [Timed UART](examples/timed-uart.md)
- [Asynchronous FIFO](examples/async-fifo.md)
- [Tri-state Bus](examples/tri-state-bus.md)

# Reference

- [Overview](reference.md)
- [Configuration](reference/configuration.md)
- [Environment Variables](reference/environment-variables.md)
- [Generated Types](reference/generated-types.md)
- [Execution Order](reference/execution-order.md)
- [Artifact Layout](reference/artifact-layout.md)
- [Compatibility](reference/compatibility.md)
- [Diagnostics](reference/diagnostics.md)
- [Terminology](reference/terminology.md)
- [Limitations](reference/limitations.md)

# Development

- [Overview](developer-guide.md)
- [Contributing](contributing.md)
- [Setup](development/setup.md)
- [Common Commands](development/common-commands.md)
- [Architecture](development/architecture.md)
- [Crate Map](development/crate-map.md)
- [Build Pipeline](development/build-pipeline.md)
- [Code Generation](development/code-generation.md)
- [Runtime](development/runtime.md)
- [Scheduler Architecture](development/scheduler-architecture.md)
- [Coverage Architecture](development/coverage-architecture.md)
- [Macro Architecture](development/macro-architecture.md)
- [FFI And Safety](development/ffi-and-safety.md)
- [Errors And Diagnostics](development/errors-and-diagnostics.md)
- [Testing](development/testing.md)
- [Fixtures](development/fixtures.md)
- [Examples](development/examples.md)
- [Documentation](development/documentation.md)
- [Benchmarking](development/benchmarking.md)
- [Compatibility](development/compatibility.md)
- [Releases](development/releases.md)

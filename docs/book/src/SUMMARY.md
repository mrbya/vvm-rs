# Summary

- [Introduction](introduction.md)
- [Why VVM?](why-vvm.md)
- [How VVM Fits Into HDL Verification](how-vvm-fits-into-hdl-verification.md)
- [How VVM Works](how-vvm-works.md)
- [Rust Essentials For HDL Engineers](rust-essentials-for-hdl-engineers.md)
- [Installation](installation.md)
- [Quick Start](quick-start.md)

# Concepts

- [Verification Workflow](concepts/verification-workflow.md)
- [DUT And Generated Wrapper](concepts/dut-and-wrapper.md)
- [Transactions](concepts/transactions.md)
- [Sequences](concepts/sequences.md)
- [Reference Models](concepts/reference-models.md)
- [Scoreboards](concepts/scoreboards.md)
- [Clocks And Time](concepts/clocks-and-time.md)
- [Failures And Results](concepts/failures-and-results.md)

# Guide

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
- [Functional Coverage](guide/functional-coverage.md)
- [Using cargo-vvm](guide/using-cargo-vvm.md)
- [Generated Type Mapping](guide/generated-type-mapping.md)
- [Execution Order](guide/execution-order.md)
- [Diagnostics And Troubleshooting](guide/troubleshooting.md)
- [Compatibility And Limitations](guide/compatibility-and-limitations.md)

# API Guide

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

# Examples

- [Overview](examples.md)
- [Counter](examples/counter.md)
- [Synchronous FIFO](examples/sync-fifo.md)
- [Timed UART](examples/timed-uart.md)
- [Asynchronous FIFO](examples/async-fifo.md)
- [Tri-state Bus](examples/tri-state-bus.md)

# Development

- [Contributing](development/contributing.md)
- [Architecture](development/architecture.md)

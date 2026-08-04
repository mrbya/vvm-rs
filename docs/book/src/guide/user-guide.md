# User Guide Overview

The Guide section answers: "What code do I write, in what order, and how do I
debug it when it fails?"

## Recommended Path

If you are following VVM for the first time, use this section in roughly this
order:

- [Project Setup](guide/project-setup.md) and [Build Script](guide/build-script.md)
  to create the crate and generate the DUT wrapper.
- [Including The DUT](guide/including-the-dut.md) to bring the generated wrapper
  into your Rust test code.
- [Driving Inputs](guide/driving-inputs.md), [Sampling Outputs](guide/sampling-outputs.md),
  and [Creating A Testbench](guide/creating-a-testbench.md) explain the normal
  cycle-driven workflow.
- [Registering Tests](guide/registering-tests.md) and
  [Configuring Tests](guide/configuring-tests.md) explain test registration and
  runtime configuration.
- [Randomization And Replay](guide/randomization-and-replay.md),
  [Waveform Tracing](guide/waveform-tracing.md), [Multi-clock Execution](guide/multi-clock.md),
  [Timing-enabled Models](guide/timing-models.md), and
  [Bidirectional Ports](guide/bidirectional-ports.md) cover the advanced paths.

## How This Section Relates To The Rest Of The Book

- Use [Concepts](concepts/overview.md) when you need the mental model.
- Use the Guide when you need step-by-step workflow instruction.
- Use the [API Guide](api-guide/overview.md) when you already know the workflow
  and need the exact public VVM surfaces.
- Use the [Reference](reference.md) section for tables, limits, environment
  variables, and exact operational rules.

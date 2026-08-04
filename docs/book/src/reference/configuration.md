# Configuration

VVM configuration comes from three layers that solve different problems.

## Configuration Layers

| Layer | Purpose | Examples |
| --- | --- | --- |
| Build-time | Decides what the generated wrapper can do | trace support, timing-capable wrapper, Verilator selection, HDL source list |
| Test descriptor | Declares what one registered test supports | `trace`, `coverage`, `cycles`, `replay(...)` |
| Run-time | Decides what one invocation will actually do | trace directory, replay token, cycle count, coverage directory |

## Important Rule

Run-time configuration cannot enable a capability that was not generated or not
declared.

For example:

- `VVM_TRACE_DIR` is useful only when the wrapper was generated with tracing and
  the test declares `trace`.
- replay settings matter only for replay-capable tests.
- timing APIs exist only for timing-capable wrappers.

## Typical Flow

1. `build.rs` decides the wrapper feature surface.
2. `#[vvm::test(...)]` declares which run-time controls the test accepts.
3. `TestRunConfig` or `TestContext` reads those controls for one invocation.
4. environment variables override the applicable run-time values.

## Where To Look Next

- [Environment Variables](environment-variables.md) for exact names and formats.
- [Configuring Tests](../guide/configuring-tests.md) for workflow usage.
- [DutBuilder API Guide](../api-guide/dut-builder.md) for build-time options.

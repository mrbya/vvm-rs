# Diagnostics And Troubleshooting

Use this page when a VVM workflow fails and you need to narrow the problem to the
right phase quickly.

## Build Phase

### Verilator Not Found

Set `VERILATOR=/path/to/verilator` or fix `PATH` before building.

### `build.rs` Fails

Check the HDL source list, top-module name, and any build-time options before you
debug Rust test code.

Representative symptoms usually mention Verilator discovery generation or bridge
compilation rather than scoreboard behavior.

### `include_dut!` Cannot Find Generated Code

Check that the logical DUT name in `DutBuilder::new(...)` exactly matches the
argument passed to `vvm::include_dut!(...)`.

## Compile Phase

### Drive Or Sample Derives Fail

Check port names, `#[vvm(port)]` mappings, and generated Rust types against the
[Generated Type Mapping](generated-type-mapping.md) chapter.

### `#[vvm::test]` Fails

Check the accepted function form, required rustdoc description, and declared
capabilities in [Registering Tests](registering-tests.md).

### Unsupported HDL Type Shape

When the generated wrapper or derives reject a port shape, treat that as a real
mapping limitation first rather than forcing a mirror type that only appears to
fit.

## Run Phase

### Randomized Failure Is Hard To Reproduce

Use the replay token through `VVM_REPLAY`, not an unrelated seed.

### Scoreboard Mismatch

Check the cycle or time context first, then separate three questions:

1. Was the sampled observation correct?
2. Was the reference-model prediction correct?
3. Is the scoreboard policy the right comparison policy?

### No Trace Or Coverage Artifact Appears

Confirm that the test declared the needed capability and that the output
directory is writable.

### Coverage Merge Or Report Fails

Check for incompatible definition fingerprints, no-artifact runs, or report
output path problems before assuming the sampling model itself is wrong.

### Timing Run Does Not Progress

Confirm that the DUT actually scheduled future timed work and that you are using
a timing-capable build and the timing scheduler rather than the ordinary
cycle-driven path.

### Inout Behaviour Looks Wrong

Re-check the caller-owned resolution policy. Most inout problems come from the
external policy layer, not from hidden wrapper behaviour.

### cargo-vvm Reporting Failure

Treat child-test failure and `cargo-vvm` post-processing failure as separate
diagnostics. A failing child run can still leave useful coverage artifacts and
reports behind.

## When To Change Tools

If the problem is fundamentally about unsupported same-time timing semantics,
four-state behaviour, or CDC proof, the right fix may be a different tool rather
than more Rust-side code.

## Related Material

- [Configuring Tests](configuring-tests.md)
- [Execution Order](execution-order.md)
- [Generated Type Mapping](generated-type-mapping.md)
- [Compatibility And Limitations](compatibility-and-limitations.md)
- [Errors API Guide](../api-guide/errors.md)

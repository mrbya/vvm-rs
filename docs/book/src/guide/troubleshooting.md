# Troubleshooting

Use this page when a VVM workflow fails and you need to narrow the problem to the
right phase quickly.

## Build Phase

### Verilator Not Found

Set `VERILATOR=/path/to/verilator` or fix `PATH` before building.

### `build.rs` Fails

Check the HDL source list, top-module name, and any build-time options before you
debug Rust test code.

### `include_dut!` Cannot Find Generated Code

Check that the logical DUT name in `DutBuilder::new(...)` exactly matches the
argument passed to `vvm::include_dut!(...)`.

## Compile Phase

### Drive Or Sample Derives Fail

Check port names, `#[vvm(port)]` mappings, and generated Rust types against the
[Generated Types](../reference/generated-types.md) reference.

### `#[vvm::test]` Fails

Check the accepted function form, required rustdoc description, and declared
capabilities in [Registering Tests](registering-tests.md).

## Run Phase

### Randomized Failure Is Hard To Reproduce

Use the replay token through `VVM_REPLAY`, not an unrelated seed.

### No Trace Or Coverage Artifact Appears

Confirm that the test declared the needed capability and that the output
directory is writable.

### Timing Run Does Not Progress

Confirm that the DUT actually scheduled future timed work and that you are using
a timing-capable build and the timing scheduler rather than the ordinary
cycle-driven path.

### Inout Behaviour Looks Wrong

Re-check the caller-owned resolution policy. Most inout problems come from the
external policy layer, not from hidden wrapper behaviour.

## When To Change Tools

If the problem is fundamentally about unsupported same-time timing semantics,
four-state behaviour, or CDC proof, the right fix may be a different tool rather
than more Rust-side code.

## Related Material

- [Limitations](../reference/limitations.md)
- [Diagnostics](../reference/diagnostics.md)

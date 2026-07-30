# Troubleshooting

## Verilator Not Found

Set `VERILATOR=/path/to/verilator` or fix your `PATH` before building the
consumer crate.

## `include_dut!` Cannot Find Generated Code

Check that the logical DUT name passed to `DutBuilder::new` exactly matches the
name passed to `vvm::include_dut!`.

## Generated Accessors Do Not Match The HDL Shape You Expected

Check [Generated Types](../reference/generated-types.md) for supported mappings
and known unsupported shapes.

## Randomized Failure Is Hard To Reproduce

Use the replay token printed by the failing test through `VVM_REPLAY`, not an
unrelated seed.

## No Trace Or Coverage Artifact Appears

Confirm that the test declared the needed capability and that the output
directory is writable.

## Timing Run Does Not Progress As Expected

Check whether the DUT actually scheduled future timed work. Also confirm that
you are using timing mode, not a cycle-only build.

## Inout Behavior Looks Wrong

Re-check your caller-owned resolution policy. Most inout problems come from the
external policy layer, not from generated wrappers silently resolving contention.

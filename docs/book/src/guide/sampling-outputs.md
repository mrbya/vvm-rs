# Sampling Outputs

Sampling turns raw DUT outputs into an observation value.

## Why This Exists

You rarely need every visible DUT signal in every check. An observation type lets
you select the outputs that matter for the current verification task.

## Minimal Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:observation}}
```

## How To Read This

- `#[derive(Sample)]` generates output-reading glue.
- `#[vvm(dut = ...)]` selects the generated DUT wrapper.
- `#[vvm(port)]` maps the observation field to one DUT output.

## Observation Versus Raw Signals

An observation should capture the data you want to compare, not necessarily every
available HDL output. That keeps models, scoreboards, and coverage focused.

## Sampling Does Not Evaluate The DUT

Sampling reads the current wrapper state after evaluation. It does not itself
advance time, apply a clock, or recompute the DUT.

## Common Variations

- sample only a subset of outputs for one testbench;
- sample signed, packed, or aggregate outputs when the generated mapping supports
  them;
- perform further Rust-side interpretation after sampling when the semantic check
  is more meaningful than the raw bits.

## Common Mistakes

- expecting sampling to cause a clock edge or reevaluation;
- selecting too many outputs and making the observation harder to reason about;
- using unsupported or mismatched field types.

## Diagnostics

Like `Drive`, many `Sample` mapping mistakes fail at compile time, which is much
faster to debug than a late runtime mismatch.

## Related Material

- [Driving Inputs](driving-inputs.md)
- [Creating A Testbench](creating-a-testbench.md)
- [Drive And Sample API Guide](../api-guide/drive-and-sample.md)

# Driving Inputs

VVM input driving is built around stimulus transactions.

## Why This Exists

The goal is not to write one setter call per HDL signal throughout the test. The
goal is to describe one meaningful verification action and let generated glue map
that action onto DUT ports.

## Minimal Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:stimulus}}
```

## How To Read This

- `#[derive(Drive)]` generates the DUT input-driving implementation.
- `#[vvm(dut = crate::event_counter::EventCounter)]` identifies which generated
  wrapper the transaction targets.
- `#[vvm(port)]` maps a field to the DUT port with the same name.
- `#[vvm(port = "event_pulse")]` shows explicit port mapping when the Rust field
  name and HDL port name differ.

## Mental Model

Treat a stimulus transaction as a semantic step such as:

- assert reset;
- present one event pulse;
- request a write;
- present input operands.

That is usually better than mirroring every signal change separately.

## Driving And Evaluation Are Separate

Driving writes inputs onto the wrapper. It does not, by itself, advance HDL state.
The later testbench clock-and-evaluate phase determines when the DUT actually
updates.

## Common Variations

- include reset fields in the transaction when reset is part of the test intent;
- use explicit `port = "..."` mapping when Rust naming should stay domain-focused;
- fall back to manual DUT method calls when a derive is not expressive enough for
  a special case.

## Common Mistakes

- treating one transaction as a dump of every RTL signal instead of a meaningful
  verification step;
- forgetting `#[vvm(port)]` or `#[vvm(port = "...")]` on a field;
- using a Rust field type that does not match the generated port mapping;
- assuming that driving implies a clock edge.

## Diagnostics

Most `Drive` mistakes fail at compile time. Typical failures are unknown port
names, duplicate mappings, or incompatible field types.

## Related Material

- [Sampling Outputs](sampling-outputs.md)
- [Drive And Sample API Guide](../api-guide/drive-and-sample.md)
- [Generated Type Mapping](generated-type-mapping.md)

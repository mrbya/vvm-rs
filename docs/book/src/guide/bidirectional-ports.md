# Bidirectional Ports

VVM exposes top-level `inout` ports as split two-state components. The caller
owns the electrical policy.

## Minimal Generalized Example

The fixture observes the full inout state:

```rust
{{#include ../../../../tests/fixtures/docs-inout-line/src/lib.rs:observation}}
```

And resolves the line in Rust:

```rust
{{#include ../../../../tests/fixtures/docs-inout-line/src/lib.rs:resolution}}
```

## What The Generated Wrapper Gives You

For an inout such as `line`, the generated wrapper exposes the presented input,
the DUT output-enable state, and the DUT output value. VVM does not choose how to
resolve contention or floating behaviour for you.

## What You Must Decide

- what floating means in your environment;
- how contention should be diagnosed;
- whether the interface is push-pull, open-drain, or something more specialized;
- whether a bounded settling loop is needed.

## Two-state Boundary

At the Rust boundary, this integration is two-state. `X` and `Z` are not carried
through as native runtime values.

## Common Mistakes

- assuming VVM silently resolves the bus for you;
- forgetting to feed the resolved input value back into the DUT;
- interpreting a two-state boundary as a four-state electrical model.

## Related Material

- [Inout API Guide](../api-guide/inout.md)
- [Generated Type Mapping](generated-type-mapping.md)
- [Tri-state bus case study](../examples/tri-state-bus.md)

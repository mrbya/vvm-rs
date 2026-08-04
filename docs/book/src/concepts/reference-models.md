# Reference Models

A VVM reference model predicts what the DUT should do for a given input history.

## Where It Fits

The reference model sits after stimulus selection and before scoreboard
comparison.

```text
Stimulus -> DUT -> Observation
    |
    v
Reference model -> Expected observation -> Scoreboard
```

## Small Generalized Example

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:reference-model}}
```

It should model the external contract of the design, not copy the RTL
implementation mechanically. A good reference model answers questions like:

- what value should appear next;
- whether a push or pop should be accepted;
- which flags should be visible after the edge;
- what protocol frame should have been transmitted.

## Why The Abstraction Exists

The model can be stateful. That is normal. A good model describes the external
contract of the block, not its internal implementation strategy. That keeps the
check meaningful even when the RTL is optimized or reorganized.

## What Reference Models Own

- expected behavior for the observed interface;
- any state needed to predict the next externally visible result;
- conversion from stimulus history into predicted outputs.

## What Reference Models Do Not Own

- the actual DUT execution;
- comparison tolerance or mismatch formatting;
- coverage accounting;
- trace output or replay control.

## Related Material

- [Creating A Testbench](../guide/creating-a-testbench.md)
- [Models And Scoreboards API Guide](../api-guide/models-and-scoreboards.md)

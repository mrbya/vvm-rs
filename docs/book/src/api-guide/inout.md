# Inout

The inout-facing API lives under `vvm::dut` through `InoutState` and generated
split accessors.

The wrapper exposes raw state such as:

- the input value currently presented to the DUT;
- the DUT's output-enable mask;
- the DUT's proposed output value.

That is enough to implement a policy such as push-pull or open-drain resolution
in ordinary Rust code. It is intentionally not a hidden analog simulator.

In practice, the most important surface names are the generated inout accessors
plus [`InoutState`](../api/vvm/dut/struct.InoutState.html). Use the
[Bidirectional Ports](../guide/bidirectional-ports.md) chapter for the workflow
and this page for the exact public API location.

Rustdoc:

- [`InoutState`](../api/vvm/dut/struct.InoutState.html)
- [`vvm::dut`](../api/vvm/dut/index.html)

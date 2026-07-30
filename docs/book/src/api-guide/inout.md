# Inout

The inout-facing API lives under `vvm::dut` through `InoutState` and generated
split accessors.

The wrapper exposes raw state such as:

- the input value currently presented to the DUT;
- the DUT's output-enable mask;
- the DUT's proposed output value.

That is enough to implement a policy such as push-pull or open-drain resolution
in ordinary Rust code. It is intentionally not a hidden analog simulator.

Rustdoc:

- [`vvm::dut`](../api/vvm/dut/index.html)

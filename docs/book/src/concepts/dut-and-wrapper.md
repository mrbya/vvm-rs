# DUT And Generated Wrapper

VVM does not talk to Verilator through handwritten Rust bindings. Instead,
`vvm-build` generates a typed wrapper for one logical DUT name during the
consumer build.

## Where It Fits

The generated wrapper is the boundary between HDL elaboration and the Rust test
code that drives, samples, schedules, and checks the model.

```text
HDL sources -> Verilator -> generated wrapper module -> Rust verification code
```

That wrapper owns:

- construction and finalization of the native model;
- generated getters and setters for supported ports;
- tracing support when the DUT was built with tracing enabled;
- timing support when the DUT was built with timing enabled.

The wrapper does not decide your verification policy. For example:

- it does not invent transactions;
- it does not know what your scoreboard should compare;
- it does not resolve top-level inout contention for you;
- it does not automatically merge coverage artifacts.

## Why The Wrapper Exists

The wrapper gives the rest of VVM a stable typed surface. `Drive`, `Sample`,
testbenches, schedulers, and tracing all work against that Rust type instead of
against raw Verilator internals.

## What It Deliberately Does Not Do

The wrapper is not your testbench. It gives you typed access to the DUT, but it
does not invent verification policy, expected behavior, or execution strategy.

## Related Material

- [Build Script](../guide/build-script.md)
- [Including The DUT](../guide/including-the-dut.md)
- [DutBuilder API Guide](../api-guide/dut-builder.md)

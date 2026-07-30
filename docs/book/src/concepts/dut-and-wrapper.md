# DUT And Generated Wrapper

VVM does not talk to Verilator through handwritten Rust bindings. Instead,
`vvm-build` generates a typed wrapper for one logical DUT name during the
consumer build.

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

The generated wrapper is the boundary between Verilator-specific details and the
Rust verification code you write on top of it.

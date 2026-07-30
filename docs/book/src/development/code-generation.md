# Code Generation

Code generation lives in `vvm-build` because it is part of the build-time
contract, not the runtime testbench contract.

Its responsibilities include:

- normalizing Verilator metadata into stable internal models;
- mapping supported HDL shapes onto Rust-facing representations;
- generating the private C++ adapter and CXX boundary;
- generating the Rust wrapper included by the consumer.

The generated wrapper is intentionally the public result. The adapter internals
are not a supported user-facing ABI.

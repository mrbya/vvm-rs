# VVM-rs

VVM-rs is a Rust-first verification framework for Verilator-generated RTL models. `vvm-build` invokes Verilator during a consumer build, generates the C++ and CXX boundary, and exposes a typed Rust DUT. The `vvm` facade provides clocks, driving, sampling, testbenches, reference models, scoreboards, replay, tracing, timing scheduling, and functional coverage.

It is intended for deterministic RTL verification driven from ordinary Rust tests. It is not a replacement for a four-state HDL simulator, UVM interoperability layer, or asynchronous runtime. Verilator execution is two-state; Rust-visible ports cannot represent HDL `X` or `Z` values.

Start with [Getting started](getting-started.md), then follow the maintained [example ladder](examples.md). Exact API contracts live in the [API reference](reference.md#api-reference).

The published site includes build channel and toolchain information in [`build-info.json`](build-info.json) when assembled by `just docs-site`.

# Reference

## API reference

The public API reference is published with this book: [vvm](../api/vvm/index.html), [vvm-build](../api/vvm_build/index.html), [vvm-core](../api/vvm_core/index.html), and [vvm-macros](../api/vvm_macros/index.html). It contains exact signatures, errors, invariants, source navigation, and API examples.

## Compatibility

VVM uses Rust edition 2024 and MSRV 1.87.0. Native verification support is Linux-focused, uses C++17 for ordinary models, and requires coroutine support for timing models. CI validates Verilator 5.000 as the minimum and 5.050 as current. Public alpha APIs may change before a stable release.

## Terminology

A DUT is the generated device wrapper. A stimulus drives inputs; an observation samples outputs. A cycle is external clock-driven execution. A timing slot is an internally scheduled Verilator event. A coverage artifact is an immutable persisted test capture.

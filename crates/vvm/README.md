# vvm-rs

`vvm-rs` exposes the `vvm` Rust facade for Verilator-backed RTL verification.

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }
```

Use `vvm-build` from `build.rs`, include generated DUTs with `vvm::include_dut!`, and author tests with the facade's DUT, testbench, timing, randomization, and coverage modules. Start with the [getting-started guide](https://byacrates.gitlab.io/vvm-rs/getting-started.html), browse [examples](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples), and see the [API reference](https://byacrates.gitlab.io/vvm-rs/api/vvm/).

VVM requires Rust 1.87.0 or newer, Verilator, and a Linux C++ toolchain for native DUT builds. This is an alpha API.

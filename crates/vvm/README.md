# vvm-rs

`vvm-rs` is the public facade crate for VVM. Normal user code depends on it as
`vvm`.

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }
```

Use `vvm-build` in `build.rs`, then include the generated DUT wrapper with
`vvm::include_dut!` in test code.

## What The Facade Provides

- generated-DUT inclusion through `include_dut!`
- typed `Drive`, `Sample`, and `Clock` derives
- `Testbench`, reference models, and scoreboards
- replayable randomization and test configuration
- timing, tracing, inout, and functional-coverage APIs

## Minimal Workflow

1. run `vvm-build` from `build.rs` with a logical DUT name;
2. include that wrapper with `vvm::include_dut!(name)`;
3. define typed stimulus, observation, and clock types;
4. run a `Testbench` from a `#[vvm::test]` function.

Compact sketch:

```rust
#[cfg(test)]
mod tests {
    use vvm::prelude::*;

    vvm::include_dut!(counter);

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Drive)]
    #[vvm(dut = crate::counter::Counter)]
    struct Stimulus {
        #[vvm(port)]
        reset_n: bool,
        #[vvm(port)]
        enable: bool,
    }
}
```

## Module Overview

- `vvm::dut`: DUT, drive, sample, tracing, inout contracts
- `vvm::testbench`: cycle-driven verification workflow
- `vvm::timing`: clocks, time, and delayed-event scheduling
- `vvm::coverage`: functional coverage, artifacts, merge, and reports
- `vvm::random`: replay tokens and deterministic randomization
- `vvm::test`: runtime configuration and registered test reports
- `vvm::packed`: packed-value helpers for advanced generated types

## Support Notes

- Rust 1.87.0 or newer
- Linux-focused native support
- Verilator-backed two-state execution
- alpha API stability

## Documentation

- Quick start: <https://byacrates.gitlab.io/vvm-rs/quick-start.html>
- User workflow guide: <https://byacrates.gitlab.io/vvm-rs/user-guide.html>
- Examples: <https://byacrates.gitlab.io/vvm-rs/examples.html>
- API reference: <https://byacrates.gitlab.io/vvm-rs/api/vvm/>

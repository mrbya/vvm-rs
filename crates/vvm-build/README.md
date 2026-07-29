# vvm-build

`vvm-build` runs Verilator and generates the native bridge from a consumer build script.

```toml
[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

```rust,no_run
use vvm_build::{BuildResult, DutBuilder};

fn main() -> BuildResult<()> {
    DutBuilder::new("counter")
        .top_module("counter")
        .source("rtl/counter.sv")
        .build()
}
```

Configure tracing with `TraceOptions` and delayed HDL execution with timing mode. The generated wrapper is included through `vvm::include_dut!`. Read the [build guide](https://byacrates.gitlab.io/vvm-rs/user-guide.html), [API reference](https://byacrates.gitlab.io/vvm-rs/api/vvm_build/), and maintained [counter example](https://gitlab.com/byacrates/vvm-rs/-/tree/master/examples/counter).

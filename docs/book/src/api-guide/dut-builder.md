# DutBuilder

`vvm_build::DutBuilder` is the build-time API that turns HDL sources into a
generated Rust wrapper plus its native bridge.

Use it when you need the exact method names and option groupings after you already
understand the [Build Script](../guide/build-script.md) workflow.

The key user-facing responsibilities are:

- choose a logical DUT name;
- declare the top module and source files;
- configure include directories and other Verilator-facing inputs;
- enable tracing or timing when needed;
- run `build()`.

The logical name matters twice:

- `DutBuilder::new("counter")` chooses the generated output directory name;
- `vvm::include_dut!(counter)` includes that generated wrapper later.

Important outputs and failures:

- generated files live under `OUT_DIR` and are meant to be included, not edited;
- `BuildStage` tells you which pipeline phase failed;
- `BuildError` keeps stage context so failures are not collapsed into a generic
  "build failed" message.

Rustdoc:

- [`vvm_build`](../api/vvm_build/index.html)
- [`DutBuilder`](../api/vvm_build/struct.DutBuilder.html)
- [`DutBuilder::new`](../api/vvm_build/struct.DutBuilder.html#method.new)
- [`DutBuilder::top_module`](../api/vvm_build/struct.DutBuilder.html#method.top_module)
- [`DutBuilder::source`](../api/vvm_build/struct.DutBuilder.html#method.source)
- [`DutBuilder::sources`](../api/vvm_build/struct.DutBuilder.html#method.sources)
- [`DutBuilder::hdl_include`](../api/vvm_build/struct.DutBuilder.html#method.hdl_include)
- [`DutBuilder::trace`](../api/vvm_build/struct.DutBuilder.html#method.trace)
- [`DutBuilder::timing`](../api/vvm_build/struct.DutBuilder.html#method.timing)
- [`DutBuilder::verilator_executable`](../api/vvm_build/struct.DutBuilder.html#method.verilator_executable)
- [`DutBuilder::build`](../api/vvm_build/struct.DutBuilder.html#method.build)
- [`BuildStage`](../api/vvm_build/enum.BuildStage.html)
- [`BuildError`](../api/vvm_build/enum.BuildError.html)

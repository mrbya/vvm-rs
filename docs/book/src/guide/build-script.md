# Build Script

`build.rs` is where VVM integration begins. The build script is responsible for
declaring which HDL sources belong to the logical DUT and which features the
generated wrapper needs.

The normal steps are:

1. choose a logical DUT name;
2. set the top module;
3. add source files and include directories;
4. enable tracing or timing when needed;
5. call `build()`.

The build script should stay boring. It is configuration, not testbench logic.

See [DutBuilder](../api-guide/dut-builder.md) for the builder surface and
[Including The DUT](including-the-dut.md) for the consuming side.

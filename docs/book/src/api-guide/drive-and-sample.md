# Drive And Sample

`Drive` and `Sample` are the two core generated-port derives.

`Drive`:

- maps fields to generated input setters;
- is for stimulus types;
- does not evaluate the DUT.

`Sample`:

- maps fields to generated output getters;
- is for observation types;
- does not evaluate the DUT.

When a direct field-to-port mapping is not the clearest observation contract,
write a manual `Sample` implementation. The synchronous FIFO example uses that
approach to expose semantic status instead of only raw flags.

Rustdoc:

- [`vvm::dut`](../api/vvm/dut/index.html)
- [`vvm_macros`](../api/vvm_macros/index.html)

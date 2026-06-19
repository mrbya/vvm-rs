# Counter Example

Milestone 1 handwritten Rust/CXX/Verilator bridge.

- `reset_n` is active low.
- `count` resets to zero when `reset_n` is low.
- `count` increments by one on each rising clock edge when `enable` is high and reset is deasserted.
- `cargo run -p vvm-example-counter` performs a manual simulation through a handwritten CXX bridge and safe local Rust wrapper.

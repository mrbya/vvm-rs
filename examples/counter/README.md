# Counter Example

Milestone 0 counter scaffold.

- `reset_n` is active low.
- `count` resets to zero when `reset_n` is low.
- `count` increments by one on each rising clock edge when `enable` is high and reset is deasserted.
- The current Rust/C++ bridge is intentionally minimal and does not instantiate Verilator yet.

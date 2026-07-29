# User guide

## DUT integration

`DutBuilder` belongs in the consumer's `build.rs`. Set a logical name, top module, source files, include directories, defines, trace options, and timing mode before `build()`. Generated output is private to `OUT_DIR`; include it from test code with `vvm::include_dut!(name)`. The builder name and inclusion name must match.

`Drive` writes generated input setters and never evaluates. `Sample` reads generated output getters and never evaluates. Packed scalar ports map to typed Rust integers; signed, wide, aggregate, and unpacked generated-port behavior is documented by the generated wrapper's rustdoc. For inout ports, generated APIs expose presented input, enable mask, output value, and `InoutState`; resolution remains caller-owned.

## Testbenches

Use a `Clock` implementation to configure a single-clock `Testbench`. A cycle drives inputs, evaluates clock transitions, samples observations, predicts through the reference model, checks the scoreboard, then runs coverage observers. `ExactScoreboard` retains cycle-aware mismatches. Failure policies stop at the first failure or retain a bounded collection. Reference models are ordinary stateful Rust values; sequences are ordinary iterators.

`#[vvm::test]` generates an ordinary Rust `#[test]`, so `cargo test`, test-name filters, and `cargo nextest run` discover it normally. Unit-style VVM tests must be inside `#[cfg(test)]`; integration tests already have test-only compilation.

## Configuration and diagnostics

`VVM_REPLAY` takes precedence over `VVM_SEED`; `VVM_CYCLES`, `VVM_TRACE_DIR`, and `VVM_COVERAGE_DIR` configure supported tests. Failures report cycle, simulation time, expected and observed values where available, plus the replay token. Trace-capable tests write VCD files under `target/vvm-trace/` by default.

## Clocks and timing

`ClockScheduler` runs named clocks with deterministic same-time ordering for multi-clock designs. This remains separate from `TimingScheduler`, which advances timing-enabled Verilator models through internally scheduled future time slots. Same-time and `#0` timing scheduling are unsupported. See the [timed UART](examples.md#timed-uart) and [asynchronous FIFO](examples.md#asynchronous-fifo) examples.

## CI and troubleshooting

Run native examples with `just test-examples`; use `just test-native-fixtures` for narrow port-generation regressions. A missing Verilator executable, unsupported HDL port, absent C++ coroutine support, or mismatched `include_dut!` name fails at an explicit build stage. Do not use VVM where four-state signal fidelity is required.

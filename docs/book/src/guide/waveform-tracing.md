# Waveform Tracing

Tracing is opt-in. A test can declare trace capability, then decide where the
trace should be written through its runtime configuration.

Typical flow:

1. mark the test with `trace` support;
2. call `configure_trace(&mut dut)` before the first relevant evaluation;
3. run the test normally;
4. inspect the resulting VCD when the test fails or when you need timing detail.

If `VVM_TRACE_DIR` is unset, VVM uses a generated directory rooted at
`target/vvm-trace/`.

Tracing is especially useful in the counter, timed UART, and tri-state bus
examples because each one demonstrates a different kind of visibility problem:
state transitions, delayed protocol activity, and resolution/settling behavior.

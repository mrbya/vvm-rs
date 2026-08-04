# Environment Variables

These variables control the normal VVM run-time and build-time overrides.

| Name | Format | Default | Precedence | Affects | Invalid value behaviour | Example |
| --- | --- | --- | --- | --- | --- | --- |
| `VVM_REPLAY` | replay token string | unset | highest replay override | replay-capable tests | configuration parse failure | `VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test event_counter_random` |
| `VVM_SEED` | integer seed | unset | below `VVM_REPLAY` | seeded/replayable random tests | configuration parse failure | `VVM_SEED=0x1234 cargo test event_counter_random` |
| `VVM_CYCLES` | unsigned integer | test-defined default | run-time override | cycle-capable tests | configuration parse failure | `VVM_CYCLES=1000 cargo test event_counter_random` |
| `VVM_TRACE_DIR` | writable directory path | generated path under `target/vvm-trace/` | run-time override | trace-capable tests | trace setup failure | `VVM_TRACE_DIR=target/traces cargo test event_counter_smoke` |
| `VVM_COVERAGE_DIR` | writable directory path | generated path under `target/vvm-coverage/` | run-time override | coverage-capable tests | persistence failure retained in run diagnostics | `VVM_COVERAGE_DIR=target/vvmcov cargo test some_covered_test` |
| `VERILATOR` | executable path | `PATH` lookup | build-time override | `vvm-build` | build-stage failure when Verilator launch fails | `VERILATOR=/opt/verilator/bin/verilator cargo test` |

## Notes

- `VVM_REPLAY` is the best reproduction control because it reconstructs the exact
  pseudo-random stream expected by the sequence.
- `VVM_SEED` is useful only when the test and sequence are designed to accept a
  seed-driven run.
- `VVM_TRACE_DIR` and `VVM_COVERAGE_DIR` do not help if the test did not declare
  the matching capability.

## Related Material

- [Configuring Tests](../guide/configuring-tests.md)
- [Randomization And Replay](../guide/randomization-and-replay.md)
- [Waveform Tracing](../guide/waveform-tracing.md)

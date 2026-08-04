# Configuring Tests

VVM test configuration can come from the descriptor, explicit runtime config, and
environment variables.

## Configuration Layers

| Layer | Purpose | Examples |
| --- | --- | --- |
| Build-time | Decides what the generated wrapper can do | trace support timing-capable wrapper Verilator selection HDL source list |
| Test descriptor | Declares what one registered test supports | `trace` `coverage` `cycles` `replay(...)` |
| Run-time | Decides what one invocation will actually do | trace directory replay token cycle count coverage directory |

Run-time configuration cannot enable a capability that was not generated or not
declared.

## Minimal Code Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:test}}
```

The quick-start test accepts `&TestRunConfig`, which is the runtime view of the
current execution settings.

## Main Controls

Common controls include:

- cycle limits;
- replay token or seed;
- trace directory;
- coverage directory;
- capability validation.

## Environment Variables

| Name | Format | Default | Precedence | Affects | Invalid value behaviour | Example |
| --- | --- | --- | --- | --- | --- | --- |
| `VVM_REPLAY` | replay token string | unset | highest replay override | replay-capable tests | configuration parse failure | `VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test event_counter_random` |
| `VVM_SEED` | integer seed | unset | below `VVM_REPLAY` | seeded or replayable random tests | configuration parse failure | `VVM_SEED=0x1234 cargo test event_counter_random` |
| `VVM_CYCLES` | unsigned integer | test-defined default | run-time override | cycle-capable tests | configuration parse failure | `VVM_CYCLES=1000 cargo test event_counter_random` |
| `VVM_TRACE_DIR` | writable directory path | generated path under `target/vvm-trace/` | run-time override | trace-capable tests | trace setup failure | `VVM_TRACE_DIR=target/traces cargo test event_counter_smoke` |
| `VVM_COVERAGE_DIR` | writable directory path | generated path under `target/vvm-coverage/` | run-time override | coverage-capable tests | persistence failure retained in run diagnostics | `VVM_COVERAGE_DIR=target/vvmcov cargo test some_covered_test` |
| `VERILATOR` | executable path | `PATH` lookup | build-time override | `vvm-build` | build-stage failure when Verilator launch fails | `VERILATOR=/opt/verilator/bin/verilator cargo test` |

`VVM_REPLAY` is the best reproduction control because it reconstructs the exact
pseudo-random stream expected by the sequence.

## Typical Invocations

Default run:

```bash
cargo test event_counter_smoke
```

Trace-enabled run:

```bash
VVM_TRACE_DIR=target/quick-start-traces cargo test event_counter_smoke
```

Replay-driven run for a replay-capable test:

```bash
VVM_REPLAY=chacha8-v1:0123456789abcdef cargo test some_random_test
```

Cycle-limited run for a capability-enabled test:

```bash
VVM_CYCLES=1000 cargo test some_random_test
```

Coverage-artifact run:

```bash
VVM_COVERAGE_DIR=target/vvm-coverage cargo test some_covered_test
```

## Precedence And Reproducibility

For replay-capable tests, replay information is the most important control for
reproducibility because it reconstructs the pseudo-random stream directly.

When an environment value is invalid, treat that as a configuration error first,
not a DUT bug.

## Common Mistakes

- setting trace or coverage directories for a test that did not declare the
  matching capability;
- confusing a seed with a replay token;
- expecting environment changes to matter when the test descriptor does not
  expose that capability.

## Related Material

- [Registering Tests](registering-tests.md)
- [Randomization And Replay](randomization-and-replay.md)
- [Waveform Tracing](waveform-tracing.md)
- [Functional Coverage](functional-coverage.md)
- [Configuration And Context API Guide](../api-guide/configuration-and-context.md)

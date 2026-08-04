# Configuring Tests

VVM test configuration can come from the descriptor, explicit runtime config, and
environment variables.

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

The exact accepted values are listed in the
[Environment Variables Reference](../reference/environment-variables.md).

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
- [Configuration And Context API Guide](../api-guide/configuration-and-context.md)
- [Environment Variables Reference](../reference/environment-variables.md)

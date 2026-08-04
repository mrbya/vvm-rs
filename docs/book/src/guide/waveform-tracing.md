# Waveform Tracing

Tracing is opt-in at both build time and run time.

## Minimal Pattern

The build script must generate a trace-capable wrapper:

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/build.rs:build-script}}
```

And the registered test must declare and configure tracing:

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:test}}
```

## Lifecycle

1. generate a trace-capable wrapper in `build.rs`;
2. declare `trace` on the test;
3. call `configure_trace(&mut dut)` before the run starts;
4. choose an output directory at run time if needed;
5. inspect the resulting VCD file.

## Typical Invocation

```bash
VVM_TRACE_DIR=target/quick-start-traces cargo test event_counter_smoke
```

## Common Missing-Trace Causes

- the wrapper was not built with trace support;
- the test did not declare the `trace` capability;
- `configure_trace(&mut dut)` was not called;
- the output directory is not writable.

## Operational Notes

- VCD files are useful for both passing and failing runs.
- Larger traces cost time and disk space.
- Use focused test filters when collecting traces in CI or locally.

## Related Material

- [Build Script](build-script.md)
- [Tracing API Guide](../api-guide/tracing.md)
- [Environment Variables Reference](../reference/environment-variables.md)

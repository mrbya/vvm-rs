# Registering Tests

VVM uses `#[vvm::test]` to integrate with normal Rust test discovery.

## Minimal Pattern

```rust
{{#include ../../../../tests/fixtures/docs-quick-start/src/lib.rs:test}}
```

## Relationship To `#[test]`

`#[vvm::test]` is not a replacement for Cargo's test system. It generates the
registration glue so the test still participates in ordinary commands such as:

```bash
cargo test event_counter_smoke
cargo nextest run event_counter_smoke
```

## Capability Arguments

Arguments such as `trace`, `coverage`, `cycles`, and `replay(...)` declare which
runtime features the test supports. VVM uses that information to validate the
run configuration and expose the right controls.

## Accepted Inputs And Return Types

Common forms use `&TestRunConfig` or `&mut TestContext` as the argument and
return `Result<TestResult<...>, Error>`.

Use `TestRunConfig` when you need run settings such as trace configuration but do
not need to capture extra runtime state manually. Use `TestContext` when the test
needs richer harness interaction.

## Intentional Failure Tests

If you keep a deliberately failing demonstration test, mark it ignored so it does
not break the default suite.

## Common Mistakes

- forgetting a rustdoc description on a VVM test function;
- declaring a capability but not using the corresponding configuration path;
- assuming VVM tests cannot be filtered or run under `nextest`.

## Related Material

- [Configuring Tests](configuring-tests.md)
- [Test Attribute API Guide](../api-guide/test-attribute.md)

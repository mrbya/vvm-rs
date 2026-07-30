# Test Attribute

`#[vvm::test]` is the user-facing test registration macro.

Common capabilities declared on the attribute include:

- `trace`
- `coverage`
- `cycles`
- `replay(...)`

The attribute chooses what runtime configuration the generated wrapper should
support. It does not replace Cargo test discovery.

Use `TestRunConfig` parameters for configuration-only tests. Use `TestContext`
parameters when you need coverage capture or richer retained diagnostics.

Rustdoc:

- [`vvm`](../api/vvm/index.html)
- [`vvm::test`](../api/vvm/test/index.html)

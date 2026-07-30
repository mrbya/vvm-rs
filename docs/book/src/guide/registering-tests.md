# Registering Tests

`#[vvm::test]` turns a VVM test function into an ordinary Rust test that Cargo
can discover.

This is an important design choice: VVM does not replace the Rust test harness.
You still use normal `cargo test` and `cargo nextest run` commands, filters,
package selection, and ignored-test handling.

Common patterns:

- `fn test(config: &TestRunConfig) -> ...` for configuration-aware tests;
- `fn test(context: &mut TestContext) -> ...` when you need coverage capture or
  richer retained diagnostics.

Unit-style VVM tests belong inside `#[cfg(test)]`. Integration tests under
`tests/` are already test-only crates.

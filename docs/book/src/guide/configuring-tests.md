# Configuring Tests

`TestRunConfig` and `TestContext` let tests opt into environment-driven runtime
configuration without hard-coding paths or replay tokens in the test body.

The common configurable pieces are:

- replay token;
- random seed fallback;
- cycle count override;
- trace output directory;
- coverage artifact directory.

Use `TestRunConfig` when a test only needs configuration. Use `TestContext`
when the test also needs to capture coverage or retain richer diagnostics.

See [Environment Variables](../reference/environment-variables.md) for the full
reference table.

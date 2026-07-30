# Configuration And Context

`TestRunConfig` exposes runtime configuration such as replay tokens, cycle
limits, trace directories, and coverage directories.

`TestContext` wraps that configuration and additionally owns test-scoped capture
facilities such as functional-coverage persistence.

Use `TestRunConfig` when your test does not need coverage capture. Use
`TestContext` when it does. That distinction keeps simple tests lightweight.

Rustdoc:

- [`vvm::test`](../api/vvm/test/index.html)

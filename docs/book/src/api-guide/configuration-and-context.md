# Configuration And Context

`TestRunConfig` exposes runtime configuration such as replay tokens, cycle
limits, trace directories, and coverage directories.

`TestContext` wraps that configuration and additionally owns test-scoped capture
facilities such as functional-coverage persistence.

Use `TestRunConfig` when your test does not need coverage capture. Use
`TestContext` when it does. That distinction keeps simple tests lightweight.

Typical `TestRunConfig` responsibilities:

- read replay and cycle overrides;
- open trace output when tracing is enabled;
- expose the run's effective configuration to the test.

Typical `TestContext` responsibilities add:

- coverage capture;
- richer test-scoped runtime coordination.

Rustdoc:

- [`TestRunConfig`](../api/vvm/test/struct.TestRunConfig.html)
- [`TestRunConfig::replay_token_or`](../api/vvm/test/struct.TestRunConfig.html#method.replay_token_or)
- [`TestRunConfig::cycles_or`](../api/vvm/test/struct.TestRunConfig.html#method.cycles_or)
- [`TestRunConfig::configure_trace`](../api/vvm/test/struct.TestRunConfig.html#method.configure_trace)
- [`TestContext`](../api/vvm/test/struct.TestContext.html)
- [`TestContext::config`](../api/vvm/test/struct.TestContext.html#method.config)
- [`TestContext::capture_coverage`](../api/vvm/test/struct.TestContext.html#method.capture_coverage)
- [`TestContext::coverage_session`](../api/vvm/test/struct.TestContext.html#method.coverage_session)

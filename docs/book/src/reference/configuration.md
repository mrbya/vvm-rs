# Configuration

Configuration comes from three places:

- build-time `DutBuilder` configuration;
- test runtime configuration through `TestRunConfig` and `TestContext`;
- environment-variable overrides.

Precedence is deliberate. Build-time configuration decides what the generated
wrapper can do. Runtime configuration decides what a specific test run will do.

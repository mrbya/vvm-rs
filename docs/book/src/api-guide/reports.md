# Reports

User-facing reports come from two related areas:

- test results and diagnostics under `vvm::test` and `vvm::testbench`;
- functional-coverage reports under `vvm::coverage::report`.

Both are structured first and rendered later. That design preserves context such
as mismatch values, replay tokens, coverage provenance, and source error chains.

Rustdoc:

- [`vvm::test`](../api/vvm/test/index.html)
- [`vvm::coverage::report`](../api/vvm/coverage/report/index.html)

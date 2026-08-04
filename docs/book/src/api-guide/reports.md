# Reports

User-facing reports come from two related areas:

- test results and diagnostics under `vvm::test` and `vvm::testbench`;
- functional-coverage reports under `vvm::coverage::report`.

Both are structured first and rendered later. That design preserves context such
as mismatch values, replay tokens, coverage provenance, and source error chains.

Rustdoc:

- [`DetailedTestReport`](../api/vvm/test/struct.DetailedTestReport.html)
- [`TestRun`](../api/vvm/test/struct.TestRun.html)
- [`TestSummary`](../api/vvm/test/struct.TestSummary.html)
- [`CoverageReport`](../api/vvm/coverage/report/struct.CoverageReport.html)
- [`CoverageReportOptions`](../api/vvm/coverage/report/struct.CoverageReportOptions.html)
- [`CoverageBinDetail`](../api/vvm/coverage/report/enum.CoverageBinDetail.html)

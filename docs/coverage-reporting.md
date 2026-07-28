# Functional coverage reporting

`CoverageReport` is explicit offline post-processing over one validated
`CoverageMerge`. It neither reads artifacts nor changes test execution.

```rust
use vvm::coverage::{merge::CoverageMerge, report::CoverageReport};

let merged = CoverageMerge::read_from("coverage.vvmcov-merged.json")?;
let report = CoverageReport::new(&merged);

std::fs::write("coverage.vvmcov.txt", report.to_text())?;
std::fs::write("coverage.vvmcov.html", report.to_html())?;
println!("{}", report.gitlab_metric());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Default options include input provenance, show uncovered normal and cross bins
plus hit illegal bins, and hide definition fingerprints. Use
`CoverageReportOptions` with `CoverageBinDetail::None`, `Uncovered`, or `All`
to choose bin detail.

Exact `CoverageRatio` counts remain authoritative. Percentages use integer
fixed-point basis points, half-up rounding, and always have two decimals. An
incomplete ratio is clamped to `99.99%`, so only a complete ratio displays
`100.00%`. Empty ratios display `0.00%` defensively.

Text reports contain merge metadata, input provenance, groups, items, counters,
selected bins, and finish with `VVM functional coverage: <percentage>`. HTML
contains the same data in an accessible self-contained HTML5 document with
inline CSS, escaped dynamic values, no JavaScript, and no external resources.

The GitLab metric is informational. VVM does not fabricate Cobertura or
JaCoCo source-line data and does not apply a pass/fail threshold.

```yaml
functional-coverage:
  stage: report
  script:
    - ./target/release/my-coverage-report
  coverage: '/^VVM functional coverage: \d+\.\d{2}%$/'
  artifacts:
    when: always
    paths:
      - coverage.vvmcov-merged.json
      - coverage.vvmcov.txt
      - coverage.vvmcov.html
```

The expression extracts the overall metric from job logs. The HTML report is a
normal job artifact and works directly from the local filesystem.
# Suite orchestration

`CoverageReport` remains the library renderer for custom tools. For normal
suite execution, use [`cargo vvm coverage`](coverage-orchestration.md), which
discovers artifacts, uses `CoverageMerge`, and renders this unchanged format.

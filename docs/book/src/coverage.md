# Functional coverage

VVM functional coverage is Rust-native and explicit. It is distinct from source-line coverage: source coverage remains a CI artifact, while this `/coverage/` namespace documents transaction intent.

`Coverpoint<T>` uses normal, ignore, and illegal `Bin`s for exact values, value sets, and inclusive ranges. Illegal bins take precedence, then ignore bins, then normal bins; all matching bins in the selected category are recorded. `Cross2` consumes normal-bin identities only, creates deterministic row-major cross bins, and skips ignored or unmatched axes. Typed models use `#[derive(vvm::Coverage)]`, `ObservedCycle`, and a `TestContext`; sampling occurs after a successful observation and does not drive or evaluate the DUT.

Each completed capture writes an immutable schema-v1 `.vvmcov.json` artifact with provenance and a structural definition fingerprint. Sessions preserve per-test state. `CoverageMerge` merges compatible artifacts by instance path under an explicit status policy; passed-only is the default, sums are checked, and output ordering is deterministic. `CoverageReport` produces deterministic text, self-contained HTML, uncovered-bin detail, and the final GitLab metric line.

Use `cargo vvm coverage` for the offline workflow. It preserves the child test status, reports its own failures separately, and handles no-artifact runs explicitly. The versioned artifact contract remains available at [schema v1](../../coverage-json-v1.md); legacy coverage files remain compatibility references.

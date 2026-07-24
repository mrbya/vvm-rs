# Coverage Merging

`CoverageMerge` combines validated per-test `.vvmcov.json` artifacts as an
explicit offline operation. Test execution never updates a shared merge.

```rust
use vvm::{CoverageMerge, CoverageMergePolicy};

let merged = CoverageMerge::from_files(
    CoverageMergePolicy::passed_only(),
    artifact_paths,
)?;

merged.write_to("target/coverage/combined.vvmcov-merged.json")?;
```

Merged documents use the `.vvmcov-merged.json` suffix and schema-v1 format
`vvm-functional-coverage-merge`. They retain provenance for every supplied
artifact, including artifacts excluded by the selected policy.

Groups merge only when their exact hierarchical `instance_path` matches.
Different paths remain independent even when their definitions are identical.
Groups at one path must have equal definition names, revisions, fingerprints,
and ordered structural data. Fingerprints are a compatibility check, not a
substitute for structural validation.

The default `passed_only` policy contributes only passed tests.
`passed_and_failed` also contributes failed tests, and `all` contributes every
status. Excluded artifacts remain in provenance but add neither counters nor
groups and do not participate in compatibility checks.

Merging sums raw runtime counters with checked arithmetic, then recomputes bin,
item, group, and merge coverage. Consequently, hits from separate tests can
meet a threshold only after merge. Persisted summaries are never summed.

Inputs are canonically ordered and merged groups are ordered lexicographically
by instance path, making results and JSON independent of input order. Identical
artifact content at distinct paths intentionally contributes more than once;
the same input path supplied twice is rejected. Text and HTML reports are
future consumers of this merged model.

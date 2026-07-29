# cargo-vvm

`cargo-vvm` is the coverage orchestration command. It remains documented in this book rather than rustdoc because it is binary-only.

Install from the repository during alpha evaluation with `cargo install --path crates/cargo-vvm --locked`. Run a child command after `coverage`, for example:

```text
cargo vvm coverage --output target/vvm-coverage --name counter -- test -p vvm-example-counter counter_
```

The command collects per-test artifacts, writes a deterministic merged JSON document, text report, and self-contained HTML report below the selected output directory. Output must be a dedicated directory; retries are not supported because an ambiguous child retry would corrupt provenance. Child test failure is preserved after reporting where artifacts exist. Reporting failure is a distinct command failure, and a no-artifact run is reported rather than silently treated as coverage.

Choose passed-only, all-completed, or explicit merge policy as required by the CLI. Bin detail controls report verbosity. Provenance and fingerprints are retained to diagnose incompatible definitions. The final text line is compatible with the GitLab coverage regex used by this repository.

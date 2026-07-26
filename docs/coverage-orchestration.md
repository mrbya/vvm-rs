# Coverage Execution Orchestration

`cargo-vvm` turns isolated VVM test artifacts into one suite report without
changing test-process behavior. Install it with `cargo install cargo-vvm`, then
run `cargo vvm coverage`. Direct invocation as `cargo-vvm coverage` is also
supported.

The default child is `cargo test --workspace`. After `--`, pass the Cargo
subcommand without the word `cargo`:

```bash
cargo vvm coverage -- test -p my-package one_test
cargo vvm coverage -- nextest run --workspace
cargo vvm coverage -- +nightly test --workspace
cargo vvm coverage -- test --manifest-path nested/Cargo.toml
```

Only `test ...` and `nextest run ...` are accepted. `nextest` retries are set to
zero, and explicit retry values other than zero are rejected because artifacts
cannot identify retry attempts.

Without `--output`, a unique directory is created beneath Cargo's target
directory. An explicit output directory must be absent or empty. `--name`
selects the portable base filename. The output contains `artifacts/`, merged
JSON, a text report, and self-contained HTML. `--merge-policy` accepts
`passed-only`, `passed-and-failed`, and `all`; `--bin-detail` accepts `none`,
`uncovered`, and `all`. `--no-inputs` hides provenance and `--fingerprints`
shows definition fingerprints.

Child stdin, stdout, and stderr remain live. Artifacts are merged and reported
even after a failed child test. A successful child plus reporting failure exits
2; a failed child always retains its original exit code. A successful child
that produces no artifacts exits 2. The complete text report is printed after
the child, with `VVM functional coverage: <percentage>` as its final line.

For GitLab, retain reports even when tests fail:

```yaml
functional-coverage:
  stage: test
  script:
    - >-
      cargo vvm coverage --output target/vvm-functional-coverage --name coverage
      -- nextest run --profile ci --workspace
  coverage: '/^VVM functional coverage: \d+\.\d{2}%$/'
  artifacts:
    when: always
    paths:
      - target/vvm-functional-coverage/
```

This is functional coverage only. No Cobertura or source-line document is
fabricated. For custom tooling, `CoverageMerge` and `CoverageReport` remain
available as library APIs.

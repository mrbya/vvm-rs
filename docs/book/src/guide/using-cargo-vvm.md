# Using cargo-vvm

`cargo-vvm` is the repository-level command for VVM functional-coverage
collection, offline merge, and report generation.

Use it when per-test VVM coverage capture already works and you now want one
command that runs the child test job, gathers artifacts into an isolated output
directory, merges them, and renders reports.

## When You Need It

You do not need `cargo-vvm` to write a coverage model. You need it when you want
suite-level merged results.

Typical reasons to use it:

- collecting artifacts from many covered tests;
- producing merged JSON, text, and HTML reports for CI or review;
- enforcing one explicit merge policy for a whole run;
- preserving child exit status while still keeping useful coverage outputs.

## Installation

```bash
cargo install cargo-vvm
```

## Basic Command Form

```text
cargo vvm coverage [OPTIONS] -- [test ... | nextest run ...]
```

The child command after `--` excludes the leading `cargo`.

Supported child forms are:

- `test ...`
- `nextest run ...`

If no child command is supplied, `cargo-vvm` falls back to `test --workspace`.

## Normal Workflow

Start with the common `cargo test` path:

```bash
cargo vvm coverage --name fifo -- test -p fifo-verification
```

This command:

1. runs `cargo test -p fifo-verification` as the child process;
2. points `VVM_COVERAGE_DIR` at a dedicated per-run artifact directory;
3. collects every per-test VVM coverage artifact produced by that child run;
4. merges compatible artifacts under the selected merge policy;
5. writes merged JSON, text, and HTML outputs;
6. prints the text report to stdout and preserves the child exit status policy.

If you prefer nextest, the supported form is:

```bash
cargo vvm coverage --name fifo -- nextest run -p fifo-verification
```

## Child-command Restrictions

`cargo-vvm` is intentionally strict about the child command because provenance
and artifact ownership matter.

- The child command must be a Cargo `test` form or `nextest run`.
- Nextest retries are disabled so one artifact still maps cleanly to one test run.
- The output directory must be absent or empty before the run starts.
- The report base name must be a portable ASCII filename.

These restrictions keep post-processing deterministic and diagnostics readable.

## Output Directory And Layout

If you do not pass `--output`, `cargo-vvm` creates a fresh directory under
`target/vvm-coverage/` with a generated run name.

If you do pass `--output`, that directory becomes the exact root for the run and
must be dedicated to that invocation.

Representative output tree:

```text
target/vvm-coverage/run-1722770000000-pid-12345-0/
|-- artifacts/
|   |-- fifo_smoke.vvm-coverage.json
|   `-- fifo_random.vvm-coverage.json
|-- fifo.vvm-coverage.json
|-- fifo.txt
`-- fifo.html
```

The important outputs are:

- `artifacts/`: per-test coverage documents captured by the child run;
- `<name>.vvm-coverage.json`: merged machine-readable JSON;
- `<name>.txt`: rendered text report;
- `<name>.html`: self-contained HTML report.

## Merge Policies

`--merge-policy` controls which test outcomes contribute.

- `passed-only`: only successful tests contribute;
- `passed-and-failed`: successful and failed tests contribute;
- `all`: every test status contributes.

Choose the policy deliberately. Some teams want only passing regressions to count
toward the final number. Others want failure-time coverage retained because it
still reveals explored behavior.

## Fingerprints And Provenance Controls

Coverage definitions carry structural fingerprints so incompatible artifacts do
not merge silently.

Two options control report detail:

- `--fingerprints` shows complete definition fingerprints in rendered reports;
- `--no-inputs` omits per-test provenance from rendered reports.

Use them when you need a more compact report or when debugging compatibility and
artifact lineage.

## Bin-detail Controls

`--bin-detail` controls how much bin-level information the rendered reports show.

- `none`: summary only;
- `uncovered`: show uncovered bins;
- `all`: show every bin.

`uncovered` is the default because it keeps attention on what still needs work.

## Failure Semantics

`cargo-vvm` separates child-test status from post-processing status.

- If the child run fails, `cargo-vvm` still tries to merge and render whatever
  artifacts were produced.
- If post-processing fails after a successful child run, `cargo-vvm` exits with
  its orchestration failure status.
- If the child fails, the final exit code preserves the child failure code.
- If the child succeeds and post-processing succeeds, the command exits
  successfully.

This means a failing regression can still leave behind useful reports.

## No-artifact Behaviour

If the child command does not produce any VVM coverage artifacts,
`cargo-vvm` reports that explicitly instead of fabricating an empty merged file.

Treat that as a workflow problem to diagnose:

- the tests may not have used covered runs;
- the tests may not have enabled VVM coverage capture;
- the child command may have selected the wrong package or test set.

## Running Coverage Through cargo test

The simplest and most portable path is still `cargo test`:

```bash
cargo vvm coverage --output target/vvm-functional-coverage --name fifo -- \
    test -p fifo-verification
```

Use this when you want one exact output directory for CI artifacts.

## Running Coverage Through nextest

When your project already uses nextest, `cargo-vvm` supports:

```bash
cargo vvm coverage --output target/vvm-functional-coverage --name fifo -- \
    nextest run -p fifo-verification
```

The important limitation is still the retry rule: retries are disabled to keep
artifact provenance unambiguous.

## GitLab CI Example

```yaml
functional-coverage:
  stage: test
  script:
    - cargo vvm coverage --output target/vvm-functional-coverage --name fifo -- test -p fifo-verification
  coverage: '/^VVM functional coverage: \d+\.\d{2}%$/'
  artifacts:
    when: always
    paths:
      - target/vvm-functional-coverage/
```

The text report ends with the metric line that GitLab can extract with a regex.

## Troubleshooting

- Use a fresh output directory per run.
- Confirm the child tests actually captured VVM coverage.
- Check merge fingerprints when artifacts refuse to combine.
- Treat child-test failure and reporting failure as separate diagnostics.
- Check the child command after `--` first when `cargo-vvm` appears to do
  nothing useful.

## Command Summary

Most users only need these options regularly:

- `--output`
- `--name`
- `--merge-policy`
- `--bin-detail`
- `--no-inputs`
- `--fingerprints`

For the exact CLI contract, use `cargo vvm coverage --help`.

## Related Material

- [Functional Coverage](functional-coverage.md)
- [cargo-vvm README](../../../crates/cargo-vvm/README.md)
- [Coverage API Guide](../api-guide/coverage.md)
- [Reports API Guide](../api-guide/reports.md)

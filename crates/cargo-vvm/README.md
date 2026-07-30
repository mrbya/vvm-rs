# cargo-vvm

`cargo-vvm` is the VVM command-line tool for offline VVM functional-coverage merge
and reporting.

## Installation

```bash
cargo install cargo-vvm
```

## What It Does

`cargo-vvm` runs a child Cargo test command, collects VVM per-test coverage
artifacts, merges them under an explicit policy, and writes merged JSON, text,
and HTML reports.

## Basic Command Form

```text
cargo vvm coverage [OPTIONS] -- [test ... | nextest run ...]
```

Example with `cargo test`:

```bash
cargo vvm coverage --output target/vvm-coverage --name counter -- \
    test -p vvm-example-counter counter_
```

Example with nextest:

```bash
cargo vvm coverage --output target/vvm-coverage --name sync-fifo -- \
    nextest run -p vvm-example-sync-fifo
```

## Output Layout

The selected output directory should be dedicated to one run. `cargo-vvm`
produces:

- per-test artifacts
- merged JSON
- text report
- self-contained HTML report

## Important Behaviors

- child test exit status is preserved
- no-artifact runs are reported explicitly
- reporting failures are distinct from child failures
- merge policy is explicit
- provenance and definition fingerprints are retained
- retries are intentionally restricted to keep provenance unambiguous

## Common Options

- `--output`
- `--name`
- `--merge-policy`
- `--bin-detail`
- `--no-inputs`
- `--fingerprints`

## GitLab CI Example

```yaml
functional-coverage:
  stage: test
  script:
    - cargo vvm coverage --output target/vvm-functional-coverage --name counter -- test -p vvm-example-counter counter_
  coverage: '/^VVM functional coverage: \d+\.\d{2}%$/'
  artifacts:
    when: always
    paths:
      - target/vvm-functional-coverage/
```

## Troubleshooting

- use a fresh output directory per run
- confirm the child tests actually captured VVM coverage
- check fingerprints when merges fail
- treat child test failure and reporting failure as separate issues

## Documentation

- Book guide: <https://byacrates.gitlab.io/vvm-rs/cargo-vvm.html>
- Command reference: <https://byacrates.gitlab.io/vvm-rs/cargo-vvm/command-reference.html>

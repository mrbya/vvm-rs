# Workflow

The common workflow is:

1. run a child `cargo test` or `cargo nextest run` command that emits per-test
   VVM coverage artifacts;
2. collect those artifacts from an isolated output directory;
3. merge them under an explicit policy;
4. write merged JSON, text, and HTML reports.

Example:

```text
cargo vvm coverage --output target/vvm-coverage --name counter -- test -p vvm-example-counter counter_
```

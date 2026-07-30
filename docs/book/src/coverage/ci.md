# CI

The common CI workflow is:

1. run tests that emit per-test coverage artifacts;
2. merge those artifacts offline;
3. publish merged JSON, text, and HTML outputs;
4. expose the text metric line to GitLab if desired.

The counter, synchronous FIFO, and asynchronous FIFO examples all demonstrate
this style of workflow.

Use `cargo-vvm` when you want the repository-level command that runs the child
test command, merges artifacts, and writes reports in one step.

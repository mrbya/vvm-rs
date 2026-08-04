# cargo-vvm Overview

This page serves as the entry point to the full `cargo-vvm` guide.

`cargo-vvm` is the binary that orchestrates offline functional-coverage merge and
report generation around a child Cargo test command.

Use it when per-test coverage capture is already working and you now need merged
reports for a whole suite or CI pipeline.

Use the following chapters for the full workflow:

- [Installation](cargo-vvm/installation.md)
- [Workflow](cargo-vvm/workflow.md)
- [Command Reference](cargo-vvm/command-reference.md)
- [Output Layout](cargo-vvm/output-layout.md)
- [Merge Policies](cargo-vvm/merge-policies.md)
- [Failure Semantics](cargo-vvm/failure-semantics.md)
- [GitLab CI](cargo-vvm/gitlab.md)
- [Troubleshooting](cargo-vvm/troubleshooting.md)

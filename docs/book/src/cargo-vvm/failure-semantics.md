# Failure Semantics

`cargo-vvm` preserves child-process status. That means a failing test command can
still produce useful merged coverage outputs before the overall command exits
unsuccessfully.

Other important behaviors:

- no-artifact runs are reported explicitly;
- reporting failures are distinct from child-test failures;
- retries are intentionally restricted because they would blur provenance.

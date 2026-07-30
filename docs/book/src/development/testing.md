# Testing

The authoritative testing policy lives in
[`docs/dev/testing-strategy.md`](../../../dev/testing-strategy.md).

At a high level, the repository separates:

- unit tests for private invariants;
- public crate integration tests;
- macro compile-time tests;
- fixture workspaces for isolated consumer scenarios;
- curated examples for end-to-end public workflows;
- native fixtures for generated-port regressions.

Use the lowest layer that proves the contract you changed.

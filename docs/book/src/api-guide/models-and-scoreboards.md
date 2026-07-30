# Models And Scoreboards

Reference models implement expected behavior. Scoreboards decide whether the
prediction and observation agree.

Use `ReferenceModel` when you need stateful expected behavior. Use
`ExactScoreboard` when equality is the right comparison policy. Use retained
failure policies when you want more than one mismatch from a run.

`Mismatch` is part of the diagnostic contract. It keeps expected and observed
values available for reports instead of hiding them behind a boolean pass/fail.

Rustdoc:

- [`vvm::testbench`](../api/vvm/testbench/index.html)

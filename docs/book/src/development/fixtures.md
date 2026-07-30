# Fixtures

Fixtures under `tests/fixtures/` are for narrow isolated contracts such as:

- generated type mapping;
- timing-delay behavior;
- synthetic multi-clock scheduling;
- package or clean-consumer verification.

They are not the public learning path. They exist so narrow regressions can be
tested without bloating the curated examples.

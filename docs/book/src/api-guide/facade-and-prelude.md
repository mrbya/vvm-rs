# Facade And Prelude

The `vvm` crate groups its public surface by domain:

- `vvm::dut`
- `vvm::testbench`
- `vvm::timing`
- `vvm::coverage`
- `vvm::random`
- `vvm::test`
- `vvm::packed`

`vvm::prelude` contains the most common test-authoring imports, but it is not a
dumping ground for every public type. Coverage persistence, packed-value tools,
and advanced schedulers still use explicit imports so the common path stays
readable.

Use the prelude when authoring ordinary tests. Use explicit module imports when
you want the code to emphasize which subsystem you are using.

Rustdoc:

- [`vvm`](../api/vvm/index.html)
- [`vvm::prelude`](../api/vvm/prelude/index.html)

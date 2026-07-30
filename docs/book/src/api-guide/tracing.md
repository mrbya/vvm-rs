# Tracing

Tracing is not a separate crate. It is a capability exposed through generated
wrappers plus runtime test configuration.

The key workflow is:

- build the DUT with tracing support;
- declare a trace-capable test;
- configure trace output before the run begins.

Timing-capable and cycle-driven wrappers can both participate in tracing, but
the test still owns when tracing is opened and where it is written.

Rustdoc:

- [`vvm::dut`](../api/vvm/dut/index.html)
- [`vvm::test`](../api/vvm/test/index.html)

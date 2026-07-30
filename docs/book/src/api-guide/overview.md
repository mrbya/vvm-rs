# API Guide Overview

Rustdoc is the exact API contract. This section explains how the public surface
fits together in normal workflows.

Use it when you already understand the concepts and want to answer questions
such as:

- which facade module owns a type;
- how `DutBuilder` relates to `include_dut!`;
- what `#[vvm::test]` expects;
- when to use `TestRunConfig` versus `TestContext`;
- where timing, tracing, inout, coverage, and report APIs live.

The canonical user entry point is the `vvm` facade. Most application code should
not depend on `vvm-core` directly.

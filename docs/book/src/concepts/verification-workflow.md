# Verification Workflow

The normal VVM data flow is:

```text
sequence
   -> stimulus transaction
   -> drive generated DUT inputs
   -> evaluate DUT and clock transitions
   -> sample observation
   -> predict expected result in a reference model
   -> compare with a scoreboard
   -> record coverage
   -> report success or failure
```

Each stage has a separate owner on purpose.

- A sequence decides what traffic to generate.
- `Drive` only applies inputs.
- the DUT and clock scheduler own execution.
- `Sample` only reads outputs.
- the reference model predicts behavior.
- the scoreboard decides whether the prediction and observation agree.
- coverage records what was seen, not whether it passed.

That split keeps VVM testbenches readable and makes failure diagnostics more
useful: when something goes wrong, you can usually tell whether the problem is
input generation, execution order, expected behavior, or observed behavior.

The counter example is the smallest complete version of this cycle. The
synchronous FIFO and asynchronous FIFO examples show why the separation matters
once queues, boundaries, and multiple clocks are involved.

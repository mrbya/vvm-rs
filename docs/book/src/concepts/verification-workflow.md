# Verification Workflow

A VVM verification workflow is the ordered path from generated stimulus to a
pass, a mismatch, or a retained runtime failure.

## Where It Fits

This is the top-level pipeline that the rest of the Concepts section breaks
apart.

```text
Sequence
   |
   v
Stimulus transaction
   |
   v
Drive inputs -> DUT execution -> Sample observation
                     |                |
                     v                v
               Reference model -> Scoreboard -> Coverage -> Result
```

Each stage has a separate owner on purpose.

- A sequence decides what traffic to generate.
- `Drive` only applies inputs.
- the DUT and clock scheduler own execution.
- `Sample` only reads outputs.
- the reference model predicts behavior.
- the scoreboard decides whether the prediction and observation agree.
- coverage records what was seen, not whether it passed.

## Why The Split Exists

That split keeps VVM testbenches readable and makes failure diagnostics more
useful. When something goes wrong, you can usually tell whether the problem is
input generation, execution order, expected behavior, observed behavior, or
coverage completeness.

## What This Workflow Owns

- the lifecycle of one verification run;
- the handoff between stimulus, execution, observation, prediction, and checks;
- the diagnostic boundaries between those phases.

## What It Does Not Own

- the exact port mapping for your DUT wrapper;
- the shape of your transactions or observations;
- the comparison policy inside a scoreboard;
- the reporting and merge policy for persisted coverage artifacts.

## Related Material

- [Creating A Testbench](../guide/creating-a-testbench.md)
- [Facade And Prelude API Guide](../api-guide/facade-and-prelude.md)

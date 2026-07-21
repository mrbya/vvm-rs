# Tri-state bus example

This package demonstrates caller-owned, deterministic push-pull resolution for
an eight-bit top-level `inout` port.

## Verilator representation

With `--pins-inout-enables`, Verilator exposes three two-state model members:

- `data`: resolved input presented to the DUT.
- `data__en`: DUT per-bit drive-enable mask.
- `data__out`: DUT drive proposal.

VVM exposes semantic methods instead of these synthetic member names:

- `set_data_input(value)` writes a caller-resolved two-state value.
- `set_data(value)` is the Drive-compatible alias.
- `data_input()` returns the presented input.
- `data_output_enable()` returns the DUT per-bit drive mask.
- `data_output_value()` returns the DUT-proposed value.
- `data()` returns `vvm::InoutState` containing all three components.

None of these methods evaluates the DUT, advances time, resolves drivers, or

## External driver and floating policy

An external participant is represented by both an enable mask and a value. A
value alone cannot represent released bits. Values are meaningful only where

When neither participant drives a bit, `resolve_bus` uses its explicit
`floating_value` argument. Passing zero models pull-down; passing one models
pull-up; a caller can pass a previous value for keeper-like behavior.

## Contention and settling

The push-pull policy detects contention per bit:

```text
contention = dut_enable AND external_enable AND (dut_value XOR external_value)
```

Contention is inferred in Rust and rejected before a new input is written.
Verilator's split representation does not provide a Rust-visible `X` value.

`settle_bus` performs a bounded loop at one simulation time: evaluate, read the
DUT proposal, resolve it with the external proposal, write a changed resolved
input, and repeat until stable. A changed input requires a following evaluation
before the HDL has observed it, so success is not returned immediately after a
write. The evaluation bound prevents non-convergent combinational feedback from
looping indefinitely.

## Two-state limitation

Released bits are represented by `output_enable = 0`; Rust receives no explicit
`Z`. Conflicting drivers receive no Rust-visible `X`. Floating and contention
semantics are caller-owned policy, not analogue or drive-strength simulation.

## Open-drain adaptation

An open-drain policy can use the same raw components without becoming the

- Enabled drivers may only propose zero.
- Released drivers use `enable = 0`.
- Floating bits resolve to one through a pull-up.
- An enabled proposal of one is rejected as invalid.
- Multiple enabled zero drivers are valid.

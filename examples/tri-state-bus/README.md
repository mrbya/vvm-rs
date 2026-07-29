# Tri-state Bus

## Purpose

This specialist example demonstrates caller-owned, deterministic resolution of
an eight-bit top-level `inout` port. It is the final example in the learning
ladder because its electrical policy is intentionally explicit.

## Design Overview

```text
external enable/value --+--> [ Rust resolution policy ] --> DUT input
                           ^              |
DUT enable/value ----------+              +--> bounded evaluate/settle loop
```

## Interface And Verification Goals

The DUT controls `drive_enable` and `drive_value`; the external participant is
represented by another enable/value pair. Verification checks floating pull-up
and pull-down policies, DUT and external ownership, split ownership, matching
overlap, exact contention detection, bounded settling, and VCD timestamps.

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
settles feedback automatically.

## External driver and floating policy

An external participant is represented by both an enable mask and a value. A
value alone cannot represent released bits. Values are meaningful only where
the enable mask is set.

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
push-pull policy used by this example:

- Enabled drivers may only propose zero.
- Released drivers use `enable = 0`.
- Floating bits resolve to one through a pull-up.
- An enabled proposal of one is rejected as invalid.
- Multiple enabled zero drivers are valid.

## Resolution Truth Table

| DUT enable | External enable | Values agree | Resolved result |
| --- | --- | --- | --- |
| 0 | 0 | n/a | Explicit floating policy |
| 1 | 0 | n/a | DUT proposal |
| 0 | 1 | n/a | External proposal |
| 1 | 1 | yes | Shared value |
| 1 | 1 | no | Rust contention error |

## Project Layout And Commands

```text
tri-state-bus/
├── build.rs
├── rtl/tri_state_bus.sv
└── src/{lib,resolution,verification}.rs
```

```bash
cargo nextest run -p vvm-example-tri-state-bus
VVM_TRACE_DIR=target/tri-state-traces cargo test -p vvm-example-tri-state-bus
```

The suite passes and the trace test retains a VCD with the resolved bus phases.
There is no randomized replay workflow because the directed resolution matrix
is intentionally deterministic. Functional coverage is represented by the
directed ownership, floating, contention, and settling scenarios.

## Known Limitations

Verilator is two-state in this integration. Rust does not receive native `X` or
`Z`; released bits are `output_enable = 0`, and conflict is diagnosed by Rust.
This is not an analog, strength, or metastability model.

## Suggested Next Example

This is the specialist end of the example ladder. Refer to
[`examples/README.md`](../README.md) to revisit the progression.

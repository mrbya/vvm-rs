# Bidirectional Ports

Top-level `inout` ports are exposed as split two-state components. VVM gives you
the raw pieces needed to implement a policy; it does not invent the policy for
you.

That means the caller owns:

- how floating bits should resolve;
- how contention is diagnosed;
- whether a bus uses push-pull, open-drain, or another policy;
- whether a design needs a bounded settling loop.

The tri-state bus example is the reference workflow here. It shows how to turn
generated `input`, `output_enable`, and `output_value` pieces into a deterministic
Rust-owned resolution step.

Remember the two-state limitation: Rust never receives native HDL `X` or `Z`
values from this integration.

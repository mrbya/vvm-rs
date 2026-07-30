# Limitations

This chapter is the authoritative limitations list for VVM.

- The integration is two-state. Rust-visible ports do not carry HDL `X` or `Z`.
- Native verification support is currently Linux-focused.
- Timing mode supports delayed future slots, not same-time or `#0` scheduling.
- Multi-clock examples demonstrate deterministic execution order, not formal CDC
  or metastability proof.
- Some HDL port shapes still rely on fixture-level validation rather than a
  polished public tutorial workflow.
- Public APIs are alpha and may change before stabilization.

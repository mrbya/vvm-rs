# Limitations And Support Policy

This page is the authoritative public support and limitations summary for VVM.

## Status And Platform

- VVM is alpha software.
- Native verification support is currently Linux-focused.
- The workspace uses Rust edition 2024 with MSRV 1.87.0.
- CI verifies Verilator 5.000 as the minimum supported version and 5.050 as the
  currently tested version.

## HDL And Runtime Semantics

- The normal integration is two-state. Rust-visible ports do not carry HDL `X` or
  `Z` values.
- Bidirectional ports are exposed as caller-resolved split state, not as a hidden
  electrical simulation policy.
- Supported generated type mappings cover common public workflows, but some HDL
  shapes still rely on fixture-level validation rather than a polished general
  user path.

## Timing And Clocks

- Timing mode supports delayed future slots, not same-time or `#0` scheduling.
- Timing and ordinary cycle-driven scheduling remain separate on purpose.
- Multi-clock support demonstrates deterministic schedules, not formal CDC proof
  or metastability modelling.

## Toolchain Notes

- Timing-enabled models additionally require coroutine-capable C++ support.
- Generated wrappers, bridges, and compiled models are build artifacts under
  Cargo-managed output directories and should not be treated as source files.

## API Stability

- Public APIs may change before the first stable release.
- When a feature is not yet documented as a supported general workflow, prefer to
  validate it against fixtures and current rustdoc before committing to it in a
  large verification environment.

## Related Material

- [Compatibility](compatibility.md)
- [Generated Types](generated-types.md)
- [Timing-enabled Models](../guide/timing-models.md)

# Compatibility And Limitations

This chapter is the authoritative public support and limitations summary for VVM.

## Status And Platform

- VVM is alpha software.
- Native verification support is currently Linux-focused.
- The workspace uses Rust edition 2024 with MSRV 1.87.0.
- CI verifies Verilator 5.000 as the minimum supported version and 5.050 as the
  currently tested version.

## Toolchain Requirements

- Native integration requires Verilator and a working C++ toolchain.
- Timing-enabled models additionally require coroutine-capable C++ support.
- Generated wrappers, bridges, and compiled models are build artifacts under
  Cargo-managed output directories and should not be treated as source files.

## HDL And Runtime Semantics

- The normal integration is two-state at the Rust boundary.
- Rust-visible ports do not preserve HDL `X` or `Z` values directly.
- Bidirectional ports are exposed as caller-resolved split state, not as a hidden
  electrical simulation model.
- Supported generated type mappings cover the common public workflows, but some
  HDL shapes still rely on fixture-level validation rather than a polished general
  user path.

## Timing And Scheduling Limits

- Timing mode supports delayed future slots, not same-time or `#0` scheduling.
- Timing and ordinary cycle-driven scheduling remain separate on purpose.
- Multi-clock support demonstrates deterministic schedules, not formal CDC proof
  or metastability modeling.
- CDC verification beyond modeled schedules is outside the current VVM scope.

## Coverage Limits

- Functional coverage is explicit and user-defined; it does not infer intent from
  HDL structure automatically.
- Coverage artifact compatibility depends on stable definition fingerprints.
- Suite-level merge/report workflows assume offline artifact collection rather
  than shared mutable runtime coverage state.

## Public API Stability

- Public APIs may change before the first stable release.
- When a feature is not documented as a supported general workflow, validate it
  against current rustdoc and the maintained fixtures before building larger
  abstractions on top of it.

## Related Material

- [Generated Type Mapping](generated-type-mapping.md)
- [Timing-enabled Models](timing-models.md)
- [Bidirectional Ports](bidirectional-ports.md)
- [Functional Coverage](functional-coverage.md)

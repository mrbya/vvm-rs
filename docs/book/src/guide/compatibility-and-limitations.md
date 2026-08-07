# Compatibility And Limitations

This chapter is the authoritative public support and limitations summary for VVM.

## Status And Tested Matrix

- VVM is pre-`1.0`, with the reviewed `v0.2.0` public API frozen for this
  release cycle.

| Surface | Supported and validated for `v0.2.0` |
| --- | --- |
| Operating system | Linux only |
| Rust toolchains | MSRV `1.87.0` and the current stable Rust toolchain |
| Verilator | Minimum `5.000`; current validated CI image `5.050` |
| Default native C++ toolchain | GCC on Linux CI |
| Additional native compiler validation | Clang on Linux native fixtures |
| Ordinary native models | C++17-capable compiler |
| Timing-enabled native models | C++20 compiler with coroutine support |

The supported surface is intentionally narrow. VVM does not advertise macOS,
Windows, MSVC, or other compiler families for native verification in the
`v0.2.0` cycle because the repository does not validate them.

## Toolchain Requirements

- Native integration requires Verilator and a working C++ toolchain.
- Ordinary native builds use the existing C++17 path.
- Timing-enabled models additionally require a C++20 compiler with coroutine
  support.
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

- Patch releases in the `0.2.x` line preserve the documented supported public
  API.
- Because VVM remains pre-`1.0`, a later minor release may still make
  intentional breaking changes with changelog documentation and migration
  guidance.
- When a feature is not documented as a supported general workflow, validate it
  against current rustdoc and the maintained fixtures before building larger
  abstractions on top of it.

## Related Material

- [Generated Type Mapping](generated-type-mapping.md)
- [Timing-enabled Models](timing-models.md)
- [Bidirectional Ports](bidirectional-ports.md)
- [Functional Coverage](functional-coverage.md)

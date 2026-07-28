# Errors and diagnostics

VVM keeps ordinary domain errors close to the domain that produces them. Small,
single-file domains keep compact validation errors in the domain module. Larger
subsystems use an `error/` module tree; coverage owns all of its related errors
under `coverage/error/`. Crate-wide orchestration pipelines use a crate-level
`error.rs`, as in `vvm-build` and `cargo-vvm`.

Use `thiserror::Error` for ordinary structured runtime and build failures.
Every nested cause must be retained with `#[source]` when it is part of the
operation failure. Add accessors for stable structured context such as paths,
stages, test names, and nested typed causes; callers must not need to parse a
display string. Extensible public errors are marked `#[non_exhaustive]` where
new variants are expected.

Manual `Display` and `Error` implementations are reserved for semantics that
cannot be expressed declaratively: generic simulation errors select a
clock-specific presentation, coverage sampling renders ordered illegal-bin
lists, cross sampling varies overflow context, and the hidden test harness
formats the final Rust test report. Generated DUT wrappers remain standard
library-only and must not require consumer crates to depend on `thiserror`.

The `vvm` facade uses concise domain names while `vvm-core` preserves internal
and generated-code names. Generated code uses only `vvm::__private`; it must
not use facade aliases. Procedural macros produce compile-time `syn::Error`
diagnostics from `vvm-macros::diagnostic`, not runtime `std::error::Error`s.

`TestDiagnostic` retains owned `CoverageRuntimeError` and
`CoverageSessionError` values. `TestContext` records them without cloning or
formatting; reports format diagnostics only at the final reporting boundary.
This preserves source chains, ordering, partial coverage, and the original
verification report.

When adding an error, choose its owning domain first, retain concrete causes
and lossless paths or commands, test each variant's display and source chain,
and add a facade export only when users should name the type directly. External
input and filesystem failures must return typed errors. Panics are limited to
documented internal invariants or test-only assertions.

## Inventory

The 12.2 inventory is organized by ownership rather than one global error enum.
Compact core validation errors are `InvalidBitVectorWordCount`,
`InvalidTimeStep`, `InvalidFailureLimit`, `ParseSeedError`,
`ParseReplayTokenError`, `PackedLayoutError`, and `UnpackedArrayIndexError`.
Clock configuration is owned by `clock/error.rs`; delayed-event scheduling is
owned by `timing.rs`; registry failures are owned by `registry.rs`; and
simulation and scoreboard diagnostics remain beside their generic runtime
models.

Coverage owns `CoverageBuildError`, `CoverageSampleError`, `CrossBuildError`,
`CrossSampleError`, `CoverageGroupError`, `CoveragePersistenceError`,
`CoverageMergeError`, `CoverageSessionError`, `CoverageDefinitionError`, and
`CoverageRuntimeError` under `coverage/error/`. Its associated inspection
enums are `CoverageIoOperation`, `CoverageGroupCountKind`,
`CoverageMergeCountKind`, `CoverageMergeCounterKind`, `CoverageSessionCountKind`,
`CoverageRuntimeItemKind`, `CrossAxis`, and `CrossCounterKind`.

`vvm-build` owns the single public `BuildError` umbrella and `BuildStage`.
`cargo-vvm` retains its crate-private command error because the binary does not
provide a Rust error API. `vvm-macros::diagnostic` creates `syn::Error` values,
which are compiler diagnostics rather than runtime errors. Generated DUT
wrappers retain standard-library-only error implementations and are exposed
only through the generated ABI.

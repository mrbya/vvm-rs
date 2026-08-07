# Public API Policy

This document is the canonical `v0.2.0` compatibility policy for VVM's reviewed
public surface.

## Release-cycle promise

VVM remains pre-`1.0`. The `v0.2.0` release cycle freezes the reviewed public
surface captured in `docs/dev/api/0.2.0`.

- `0.2.x` patch releases preserve the documented supported public Rust API.
- Later pre-`1.0` minor releases may still make intentional breaking changes,
  but only with changelog entries, compatibility review, and updated baselines.
- APIs outside the documented supported surface are not covered by the patch-line
  compatibility promise.

## Compatibility domains

- Rust API compatibility is defined by the accepted facade surface and canonical
  module paths below.
- Persisted coverage compatibility is versioned JSON compatibility plus matching
  reviewed structural fingerprints; artifacts with unsupported schema versions or
  incompatible definitions are rejected instead of merged silently.
- Generated-code compatibility covers the public generated wrapper contract:
  generated module naming, supported accessor naming, trait implementations, and
  documented DUT capabilities. Internal generated implementation details remain
  free to change.
- CLI compatibility covers machine-consumed `cargo-vvm` behavior documented for
  the `v0.2.0` line: stable output file names and layout, supported child-command
  forms, merge-policy semantics, and the final `VVM functional coverage:` metric
  line.
- `vvm::__private` is never part of the compatibility promise.

## Accepted `v0.2.0` baseline

Use `docs/dev/api/0.2.0` as the accepted public API baseline for the release
line. Contributors should compare current output against that checked-in file
instead of assuming a published `v0.2.0` tag already exists.

`vvm` is the supported user facade. Canonical domains are `coverage`, `dut`,
`packed`, `random`, `test`, `testbench`, and `timing`; coverage persistence and
reporting live in `coverage::{artifact, merge, report, session, snapshot}`.

The root exports only `Clock`, `Coverage`, `Drive`, and `Sample` derives, the
`test` attribute, `include_dut!`, facade modules, and the prelude. Ordinary
types are not duplicated at the root. The previous flat facade was deliberately
removed before `v0.2.0`.

The prelude contains common test-authoring derives and traits, `Dut`, basic
testbench components, `TestContext`, and `TestRunConfig`. Packed values,
coverage primitives and reports, timing schedulers, and registry internals need
explicit imports.

`vvm::__private` is an unsupported generated-source ABI. It is public only
because macro expansions and `vvm-build` wrappers compile in downstream crates.
Users must not import it; its contents may change with generated code.

`vvm-core` is used directly only by internal orchestration packages when their
dependency boundary requires runtime models. Application code should use `vvm`.
`vvm-macros` is an implementation package; use root derives from `vvm`.
`vvm-build` generates and compiles Verilator bridges from consumer build scripts.
`cargo-vvm` orchestrates coverage artifacts and intentionally uses core models.

New root exports require a Rust-language ergonomics justification. New prelude
items must be common to ordinary test bodies. Do not create duplicate canonical
paths; generated-only support belongs in `__private`.

## Final error paths

The reviewed facade error paths are `vvm::coverage::{BuildError,
DefinitionError, GroupError, RuntimeError, SampleError, CrossBuildError,
CrossSampleError}`, `vvm::coverage::artifact::{IoOperation, PersistenceError}`,
`vvm::coverage::merge::{MergeError, MergeCountKind, MergeCounterKind}`,
`vvm::coverage::session::{SessionError, SessionCountKind}`,
`vvm::packed::{LayoutError, UnpackedIndexError, WordCountError}`,
`vvm::random::{ReplayTokenParseError, SeedParseError}`,
`vvm::test::RegistryError`, `vvm::testbench::{FailureLimitError,
SimulationError}`, and `vvm::timing::{ClockConfigurationError, SchedulerError,
TimeStepError}`.

The inventory was reviewed with `cargo public-api` for `vvm-rs`, `vvm-core`,
and `vvm-build` after the 12.2 migration. The `v0.2.0` baseline uses facade
modules as canonical user-facing APIs, coverage
submodules are advanced user-facing APIs, `__private` is generated-only, and
the former root types plus `IntoTestOutcome`, callback aliases, `NoCoverage`,
`Unconfigured`, `CoverageSpec`, `CoverageSampleSpec`, and
`unpacked_array_ordinal` are removed accidental exports. The final semver
baseline includes the canonical error paths above.

## Contributor commands

- `just api` prints the current public API inventory.
- `just api-gen` regenerates the inventory file for the current crate version.
- `just api-diff` compares the current branch against `docs/dev/api/0.2.0`.

# Public API Policy

`vvm` is the supported user facade. Canonical domains are `coverage`, `dut`,
`packed`, `random`, `test`, `testbench`, and `timing`; coverage persistence and
reporting live in `coverage::{artifact, merge, report, session, snapshot}`.

The root exports only `Clock`, `Coverage`, `Drive`, and `Sample` derives, the
`test` attribute, `include_dut!`, facade modules, and the prelude. Ordinary
types are not duplicated at the root. The previous flat facade was deliberately
removed before 0.1.0.

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
and `vvm-build` after the 12.2 migration. The v0.1.0 baseline uses facade
modules as canonical user-facing APIs, coverage
submodules are advanced user-facing APIs, `__private` is generated-only, and
the former root types plus `IntoTestOutcome`, callback aliases, `NoCoverage`,
`Unconfigured`, `CoverageSpec`, `CoverageSampleSpec`, and
`unpacked_array_ordinal` are removed accidental exports. The final semver
baseline includes the canonical error paths above.

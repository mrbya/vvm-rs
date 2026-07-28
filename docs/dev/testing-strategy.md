# Testing strategy

This document is the authoritative test-location and execution policy for VVM.
Tests prove a contract at the lowest layer that exposes that contract; they are
not kept in a module merely because that is where they were first written.

## Taxonomy

| Category | Location | Contract | Verilator | Nested Cargo |
| --- | --- | --- | --- | --- |
| Unit | Inline `#[cfg(test)]` module or sibling test module | Private invariants, parsing, checked arithmetic, state transitions, and error rendering | No | No |
| Crate integration | `crates/<crate>/tests/` | Public APIs and cross-module consumer behavior | Usually no | No |
| Compile-time | `crates/vvm-macros/tests/` | Macro expansion, typestate, and diagnostics | No | `trybuild` only |
| Fixture workspace | `tests/fixtures/` | Isolated Cargo consumers and package scenarios | Depends on fixture | Yes |
| End-to-end | Integration harnesses operating on fixtures/examples | Rust, C++, Verilator, and `cargo-vvm` workflows | Yes | Usually |

Integration tests use public APIs only. Facade tests import `vvm` and must not
import `vvm-core`. Unit tests may access private implementation only when that
access proves an implementation invariant which cannot be expressed through a
public contract.

## Baseline

Captured on 2026-07-28 with Verilator 5.048:

| Measure | Baseline |
| --- | --- |
| Nextest cases | 546 passed, 1 skipped before migration |
| Nextest duration | 27.610 seconds |
| Rust line coverage | 83.35% |
| Rust branch coverage | Unavailable: installed `cargo-llvm-cov` exposes `--branch` as unstable |
| Native tests | `vvm` RTL lint plus all `examples/*` test targets |
| Nested Cargo tests | None before this milestone |
| Compile fixtures | `vvm-macros/tests/ui/{pass,fail}` |
| Golden fixtures | `vvm-build/tests/fixtures/{codegen,verilator/5.048}` |

The baseline validation passed `just fmt --check`, `just check -- -D warnings`,
`just test`, `just doctest`, `just test-cov-ci`, `just unused`, and `just audit`.
The coverage report identified `cargo-vvm` command, metadata, output, and
process modules as currently unexecuted or only incidentally executed. They are
priority targets for dedicated public CLI integration coverage, not exclusions.

The checked-in aggregate line floor is 83%. Per-crate line floors are measured
from the post-migration profile and enforced serially from the same profile:
`vvm` 80%, `vvm-core` 82%, `vvm-build` 90%, `vvm-macros` 74%, and `cargo-vvm`
66%. They are rounded down only enough to absorb tool noise and must not be
lowered for unrelated changes. Branch output remains unavailable because the
installed `cargo-llvm-cov` exposes it as unstable; branch floors will be added
when the runner supports stable branch instrumentation.

## Inventory

The exact case-level inventory is produced by:

```text
cargo nextest list --all-features --workspace
cargo test --workspace --doc -- --list
```

This is deliberately command-derived rather than a stale hand-maintained list.
The following table classifies every discovered suite by source location; test
names within each listed module have the category shown in that row.

| Suite location | Owning crate | Current/correct category | Contract | Verilator | Nested Cargo | Cost | Duplicate action |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `crates/vvm-core/src/**/*.rs` test modules | `vvm-core` | Unit | Runtime, scheduler, packed values, replay, coverage, persistence | No | No | Low | Retain; add public workflows in `tests/` only when needed |
| `crates/vvm/src/harness.rs` | `vvm` | Unit | Environment parsing and harness persistence | No | No | Low | Retain |
| `crates/vvm/tests/public_api.rs` | `vvm` | Crate integration | Facade paths, derives, prelude, and testbench behavior | No | No | Low | Retain facade-only rule |
| `crates/vvm/tests/integration_tests.rs` | `vvm` | End-to-end | Counter assets and Verilator lint | Yes | No | Low | Keep native-only |
| `crates/vvm/tests/fixtures.rs` | `vvm` | Fixture workspace | Clean facade consumer with isolated target | No | Yes | Medium | New canonical pure consumer check |
| `crates/vvm-build/src/**/*.rs` test modules | `vvm-build` | Unit | Metadata normalization, code generation, command construction | No | No | Low | Retain golden fixture coverage |
| `crates/cargo-vvm/src/**/*.rs` test modules | `cargo-vvm` | Unit | CLI parsing, command validation, orchestration helpers | No | No | Low | Add public binary contracts under `tests/` |
| `crates/vvm-macros/src/**/*.rs` test modules | `vvm-macros` | Unit | Token parsing and expansion invariants | No | No | Low | Retain |
| `crates/vvm-macros/tests/trybuild.rs` and `tests/ui/**` | `vvm-macros` | Compile-time | Public derive and attribute expansion/diagnostics | No | `trybuild` | Medium | Preserve reviewed snapshots |
| `examples/*/src/{verification,test_cases,resolution,coverage}.rs` | Example crates | End-to-end or local unit | Native workflows and example-local reference models | Yes except pure helpers | No | Medium | Extract only infrastructure regressions; defer curation to 12.4 |
| `crates/vvm-build/tests/fixtures/codegen/**` | `vvm-build` | Golden fixture | Generated bridge/wrapper regression inputs | No | No | Low | Do not regenerate casually |
| `crates/vvm-build/tests/fixtures/verilator/5.048/**` | `vvm-build` | Golden fixture | Versioned Verilator metadata normalization | No | No | Low | Do not regenerate or reformat casually |

## Fixtures and subprocesses

Every fixture project has an explicit empty `[workspace]` table. A harness must
copy the fixture into a temporary directory before modifying it, rewrite only
the declared dependency placeholders, set a unique `CARGO_TARGET_DIR`, and
print complete stdout/stderr when Cargo fails. Fixture manifests must never use
workspace-relative paths after rewriting. Native fixtures belong in a clearly
named `native-*` directory and are run only by native or end-to-end recipes.

`pure-consumer` validates that an external project can depend on the facade
alone. `build-consumer` performs a clean Rust-CXX-Verilator build through its
own `build.rs` and public `vvm::include_dut!` invocation. `packaged-consumer`
uses only temporary extracted `cargo package` archives, patches its VVM
dependencies to those extracted contents, and runs offline. All three use the
same isolation rules.

## Commands

| Command | Scope |
| --- | --- |
| `just test-unit` | Library unit targets across the workspace |
| `just test-integration` | Public pure-Rust crate contracts |
| `just test-ui` | `trybuild` pass/fail suites |
| `just test-fixtures` | Pure clean consumer fixtures |
| `just test-fast` | Unit, integration, UI, and fixture suites |
| `just test-native` | Verilator/C++ dependent workspace tests |
| `just test-e2e` | Counter, timing, multi-clock, and inout workflows |
| `just test-package` | Local publishable crate archives without registry access |
| `just test-all` | Every test category |
| `just test-cov` | Whole-workspace line coverage with the 90% floor |
| `just test-cov-ci` | Aggregate and per-crate line coverage, with text, JSON, Cobertura, and HTML artifacts under `coverage/` |
| `just mutation` | Manual/scheduled pure-Rust mutation scope; requires `cargo-mutants` |
| `just ci` | Contributor-equivalent required gate |

`VVM_REPLAY` takes precedence over `VVM_SEED`; record replay tokens in a test
failure and use bounded case counts for any property suite. `VVM_CYCLES`,
`VVM_TRACE_DIR`, and `VVM_COVERAGE_DIR` must be passed explicitly by fixture
harnesses rather than inherited accidentally. Native commands require a
supported Verilator and C++ toolchain.

`ci/Dockerfile.verilator` builds native CI images from a versioned Verilator
source archive and requires its SHA-256 checksum. Publish a release with
`just docker-build-verilator <version> <sha256>`; the image tag is
`verilator-<version>-rust-1`. The native matrix uses only these version tags,
not the rolling developer `Dockerfile` image.

## Golden files and coverage

Review each `trybuild` `.stderr` file when a macro diagnostic intentionally
changes. Do not accept snapshots for incidental compiler wording. Keep the
Verilator 5.048 metadata and code-generation goldens byte-stable unless the
corresponding compatibility change is deliberate and reviewed.

Rust coverage includes ordinary deterministic Rust production code. The only
permitted exclusions are generated Rust in `OUT_DIR`, generated CXX bridge and
Verilator sources, and unsupported platform glue. Error paths, serializers,
formatters, and command construction are never excluded merely because they
are inconvenient. CI retains text, JSON, Cobertura XML, and HTML reports even
after a failing job.

`cargo-mutants` is installed by `just init`. The initial reproducible baseline
is `cargo-mutants` 27.1.0 over checked time arithmetic (`just mutation`), with
one job and a 60-second per-command timeout. The baseline ran 15 mutants in
29 seconds: 7 caught and 8 unviable. The initial `TimeStep::checked_add`
survivor was killed by `time_step_checked_add_preserves_non_zero_sum`. Expand
the scheduled scope to packed values, replay, coverage merge, persistence, and
CLI status propagation only after recording duration and investigating every
arithmetic survivor.

## Regression tests

For a fixed bug, add the smallest deterministic test that fails before the
fix, put it in the category matching the user-visible contract, name the
behavior rather than the issue number, and preserve the original reproducer's
seed or persisted input where applicable. Do not duplicate the same assertion
at several abstraction layers.

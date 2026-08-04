# Documentation Audit

This document records the milestone 12.5 documentation-depth audit completed on
2026-07-30. It is a maintainer planning artifact, not user-facing book content.

## Adopted Principles

The `.idea/zappy` reference project was inspected read-only for structure and
documentation progression. VVM adopts these principles without copying
Zappy-specific product language or chapter concepts:

- Keep the root `README.md` concise but self-sufficient.
- Organize the book into orientation, workflows, API guidance, examples,
  reference material, and development material.
- Prefer step-by-step workflow chapters over broad summary pages.
- Keep conceptual guidance separate from exact API reference.
- Document current limitations honestly in one authoritative location.
- Provide crate-boundary, architecture, and contributor workflow material that
  lets contributors place changes without reading the whole codebase first.
- Use navigation that lets readers move from first setup through advanced usage
  to implementation architecture.

## Current-State Findings

- The current book is structurally valid but still flat and summary-heavy:
  `introduction.md`, `getting-started.md`, `user-guide.md`, `coverage.md`,
  `cargo-vvm.md`, `examples.md`, `reference.md`, `developer-guide.md`, and
  `contributing.md` compress entire subsystems into a few paragraphs each.
- The root `README.md` still contains the richest explanations for timing,
  inout, functional coverage, environment variables, and the end-to-end VVM
  workflow.
- The package READMEs for `vvm-rs`, `vvm-build`, and `cargo-vvm` are too brief
  to stand alone on crates.io or in isolation.
- The example READMEs contain practical information that the book does not yet
  explain, especially around coverage design, timing semantics, multi-clock
  behavior, inout resolution, and CI workflows.
- Public APIs remain documented most precisely in rustdoc, but the book does not
  yet explain how the major facade modules fit together in ordinary workflows.

## Coverage Matrix

| Topic or API | Current location | Depth | Accuracy | Audience | Future authoritative location | Supporting example | Rustdoc link | Missing information | Required action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Root README overview | `README.md` | Medium | Mostly accurate | New users, crate consumers | `README.md` plus `docs/book/src/introduction.md` | `examples/counter` | `api/vvm/index.html` | Motivation, limitations, quick usage, and docs map need cleaner balance | Rewrite README and mirror detailed workflow material into the book |
| README quick start | `README.md#quick-start` | Medium | Incomplete | New users | `docs/book/src/quick-start.md` | `examples/counter` | `api/vvm/index.html`, `api/vvm_build/index.html` | Includes essential pseudocode and leaves testbench assembly implicit | Replace with source-backed quick start |
| README timing guide | `README.md#timing-enabled-models` | Medium | Accurate | Advanced users | `docs/book/src/guide/timing-models.md`, `docs/book/src/api-guide/schedulers.md`, `docs/book/src/reference/limitations.md` | `examples/timed-uart` | `api/vvm/timing/index.html` | Scheduler workflow, limits, and timing/cycle separation need full treatment | Move expanded explanation into book chapters |
| README inout guide | `README.md#bidirectional-ports` | Medium | Accurate | Advanced users | `docs/book/src/guide/bidirectional-ports.md`, `docs/book/src/api-guide/inout.md`, `docs/book/src/reference/generated-types.md` | `examples/tri-state-bus` | `api/vvm/dut/index.html` | Generated inout contract, resolution policy, contention, and settling need full guide | Expand book and trim README to summary |
| README coverage workflow | `README.md#functional-coverage` | Medium | Accurate | Users, CI maintainers | `docs/book/src/coverage/*.md`, `docs/book/src/cargo-vvm/*.md` | `examples/counter`, `examples/sync-fifo` | `api/vvm/coverage/index.html` | Coverage authoring, persistence, merging, reports, and CLI orchestration are still scattered | Split into progressive coverage and CLI chapters |
| Root README environment variables | `README.md#test-configuration` | Medium | Accurate | Users | `docs/book/src/reference/environment-variables.md` | `examples/counter` | `api/vvm/test/struct.TestRunConfig.html` | Table form, precedence, formats, and error behavior are missing | Add full environment-variable reference |
| `vvm-rs` package README | `crates/vvm/README.md` | Low | Accurate but sparse | Crate consumers | `crates/vvm/README.md` | `examples/counter` | `api/vvm/index.html` | No real workflow, module map, or support notes | Rewrite as standalone facade documentation |
| `vvm-build` package README | `crates/vvm-build/README.md` | Low | Accurate but sparse | Build-script authors | `crates/vvm-build/README.md` | `examples/counter/build.rs` | `api/vvm_build/index.html` | Missing naming contract, builder options, outputs, rebuild behavior, and failures | Rewrite as standalone build guide |
| `cargo-vvm` package README | `crates/cargo-vvm/README.md` | Low | Accurate but sparse | Coverage and CI users | `crates/cargo-vvm/README.md` | `examples/counter/README.md` | binary-only; book chapter source is `docs/book/src/cargo-vvm/*.md` | Missing command workflow, output layout, failure semantics, and troubleshooting | Rewrite as standalone command documentation |
| Introduction chapter | `docs/book/src/introduction.md` | Low | Accurate | New users | `docs/book/src/introduction.md` | `examples/counter` | `api/vvm/index.html` | Audience, maturity, non-goals, limitations, architecture components, and next-path guidance are too brief | Expand into full orientation chapter |
| Getting started chapter | `docs/book/src/getting-started.md` | Low | Accurate | New users | `docs/book/src/installation.md`, `docs/book/src/quick-start.md` | `examples/counter` | `api/vvm_build/struct.DutBuilder.html` | Prerequisites and first workflow are too compressed | Split into installation and quick-start chapters |
| User guide chapter | `docs/book/src/user-guide.md` | Low | Accurate | Users | `docs/book/src/concepts/*.md`, `docs/book/src/guide/*.md` | `examples/counter`, `examples/sync-fifo` | `api/vvm/index.html` | Entire workflow, concepts, and API surface are summarized instead of taught | Replace with concept and workflow sections |
| Coverage chapter | `docs/book/src/coverage.md` | Low | Accurate | Users, CI maintainers | `docs/book/src/coverage/*.md` | `examples/counter`, `examples/sync-fifo`, `examples/timed-uart` | `api/vvm/coverage/index.html` | No progressive authoring, coverage design advice, CI workflow, or troubleshooting | Replace with multi-chapter coverage guide |
| cargo-vvm chapter | `docs/book/src/cargo-vvm.md` | Low | Accurate | CI maintainers | `docs/book/src/cargo-vvm/*.md` | `examples/counter/README.md` | n/a | Needs installation, command reference, output tree, failure semantics, GitLab example, troubleshooting | Replace with multi-chapter CLI guide |
| Examples chapter | `docs/book/src/examples.md` | Low | Accurate | Users | `docs/book/src/examples/*.md` | all curated examples | n/a | Lacks architecture, file map, concepts, limitations, and next steps per example | Replace with full learning-ladder chapters |
| Reference chapter | `docs/book/src/reference.md` | Low | Accurate | Users | `docs/book/src/reference/*.md` | fixtures and examples | `api/vvm/index.html` | No configuration tables, env vars, generated-type mapping, diagnostics, or limitations detail | Replace with practical reference section |
| Developer guide chapter | `docs/book/src/developer-guide.md` | Low | Accurate | Contributors | `docs/book/src/development/*.md` | source tree | internal/public rustdoc | Architecture, pipelines, ownership, scheduler internals, coverage internals, macro details, and safety invariants are compressed | Replace with architecture section |
| Contributing chapter | `docs/book/src/contributing.md` | Low | Accurate | Contributors | `docs/book/src/development/contributing.md`, `docs/book/src/development/documentation.md` | `justfile` | n/a | Setup, command matrix, testing categories, docs maintenance, and PR expectations are incomplete | Expand contributor docs |
| Examples index README | `examples/README.md` | Medium | Accurate | Users | `examples/README.md` plus `docs/book/src/examples/overview.md` | all curated examples | n/a | Ladder is good but book does not reflect it fully | Preserve README and mirror learning path into book |
| Counter example README | `examples/counter/README.md` | High | Accurate | New users | local README plus `docs/book/src/quick-start.md`, `docs/book/src/examples/counter.md`, `docs/book/src/coverage/*.md` | counter source tree | `api/vvm/testbench/struct.Testbench.html` | Book lacks its practical workflow, intentional failure, and coverage orchestration detail | Reuse as source-backed primary beginner path |
| Sync FIFO example README | `examples/sync-fifo/README.md` | Medium | Accurate | Intermediate users | local README plus `docs/book/src/examples/sync-fifo.md`, `docs/book/src/coverage/*.md` | `examples/sync-fifo/src/lib.rs` | `api/vvm/testbench/trait.ReferenceModel.html` | Book lacks queue-model and boundary-behavior explanation | Add conceptual example chapter |
| Timed UART example README | `examples/timed-uart/README.md` | Medium | Accurate | Advanced users | local README plus `docs/book/src/examples/timed-uart.md`, `docs/book/src/guide/timing-models.md` | `examples/timed-uart/src/lib.rs` | `api/vvm/timing/struct.TimingScheduler.html` | Book lacks protocol reconstruction, timed execution, and manual coverage example | Add conceptual example chapter |
| Async FIFO example README | `examples/async-fifo/README.md` | Medium | Accurate | Advanced users | local README plus `docs/book/src/examples/async-fifo.md`, `docs/book/src/guide/multi-clock.md` | `examples/async-fifo/src/test_cases.rs` | `api/vvm/timing/struct.ClockScheduler.html` | Book lacks deterministic same-time ordering and direct multi-clock stepping explanation | Add conceptual example chapter |
| Tri-state bus example README | `examples/tri-state-bus/README.md` | High | Accurate | Specialist users | local README plus `docs/book/src/examples/tri-state-bus.md`, `docs/book/src/guide/bidirectional-ports.md` | `examples/tri-state-bus/src/resolution.rs` | `api/vvm/dut/enum.InoutState.html` | Book lacks general generated inout contract and resolution policy explanation | Add conceptual example and inout guide |
| Facade module overview | `crates/vvm/src/lib.rs`, `docs/dev/public-api.md` | Medium | Accurate | Crate consumers | `docs/book/src/api-guide/overview.md`, `docs/book/src/api-guide/facade-and-prelude.md` | `crates/vvm/tests/public_api.rs` | `api/vvm/index.html`, `api/vvm/prelude/index.html` | No book chapter explains module boundaries and canonical imports | Add API overview chapters |
| `vvm::include_dut!` and generated wrapper contract | rustdoc, examples, README | Medium | Accurate | Users | `docs/book/src/guide/including-the-dut.md`, `docs/book/src/api-guide/dut-builder.md`, `docs/book/src/reference/generated-types.md` | `examples/counter/build.rs` | `api/vvm/macro.include_dut.html` | Name alignment, generated methods, trace/timing variants, and generated errors need explicit guide | Add build/inclusion chapters |
| `DutBuilder`, `BuildResult`, `TraceOptions`, `BuildError`, `BuildStage` | rustdoc, README, current getting-started | Medium | Accurate | Build-script authors | `docs/book/src/guide/build-script.md`, `docs/book/src/api-guide/dut-builder.md`, `crates/vvm-build/README.md` | `examples/counter/build.rs`, `examples/timed-uart/build.rs` | `api/vvm_build/index.html` | Missing normal workflow, options, output layout, and failure stages at book and README level | Expand book and README |
| `Drive`, `Sample`, `Clock` derives | rustdoc, examples | Medium | Accurate | Users | `docs/book/src/guide/driving-inputs.md`, `docs/book/src/guide/sampling-outputs.md`, `docs/book/src/api-guide/drive-and-sample.md`, `docs/book/src/api-guide/clocks.md` | `examples/counter/src/verification.rs` | `api/vvm/index.html`, `api/vvm_macros/index.html` | Supported shapes, field attributes, and diagnostics are not collected in one place | Add derive and attribute documentation |
| `#[derive(Coverage)]` | rustdoc, counter example | Medium | Accurate | Users | `docs/book/src/coverage/typed-models.md`, `docs/book/src/api-guide/coverage.md` | `examples/counter/src/coverage.rs` | `api/vvm/index.html`, `api/vvm_macros/index.html` | Supported structure, item ordering, errors, and when to use manual coverage need book guidance | Add typed coverage chapters |
| `#[vvm::test]` and `#[vvm(...)]` attributes | rustdoc, examples | Low | Accurate | Users | `docs/book/src/guide/registering-tests.md`, `docs/book/src/api-guide/test-attribute.md` | `examples/counter/src/test_cases.rs` | `api/vvm/index.html`, `api/vvm/test/index.html` | Accepted signatures, capabilities, config handoff, and diagnostics are under-documented | Add test attribute chapters |
| `Testbench`, `ReferenceModel`, `Scoreboard`, `ExactScoreboard`, `Mismatch`, `FailurePolicy`, `ObservedCycle`, `TestResult` | rustdoc, examples, README | Medium | Accurate | Users | `docs/book/src/concepts/*.md`, `docs/book/src/guide/creating-a-testbench.md`, `docs/book/src/api-guide/testbench.md`, `docs/book/src/api-guide/models-and-scoreboards.md` | `examples/counter`, `examples/sync-fifo` | `api/vvm/testbench/index.html` | Workflow, phases, failure retention, and model guidance need deeper explanation | Add concepts and API chapters |
| `TestRunConfig`, `TestContext`, reports, diagnostics | rustdoc, README | Medium | Accurate | Users | `docs/book/src/guide/configuring-tests.md`, `docs/book/src/api-guide/configuration-and-context.md`, `docs/book/src/api-guide/reports.md`, `docs/book/src/reference/diagnostics.md` | `examples/counter/src/test_cases.rs` | `api/vvm/test/index.html` | Config precedence, artifact directories, retained diagnostics, and reporting need full guide | Add config and diagnostics chapters |
| `Seed`, `ReplayToken`, `RandomContext`, `ReplayableSequence`, `Randomize` | rustdoc, counter and fifo examples | Medium | Accurate | Users | `docs/book/src/guide/randomization-and-replay.md`, `docs/book/src/api-guide/randomization.md`, `docs/book/src/reference/environment-variables.md` | `examples/counter/src/test_cases.rs` | `api/vvm/random/index.html` | Replay precedence, deterministic sequence design, and failure reproduction need fuller treatment | Add randomization chapters |
| `SimulationTime`, `TimeStep`, `ClockScheduler`, `ClockTiming`, `CycleTiming` | rustdoc, async FIFO README | Medium | Accurate | Advanced users | `docs/book/src/concepts/clocks-and-time.md`, `docs/book/src/guide/multi-clock.md`, `docs/book/src/api-guide/clocks.md`, `docs/book/src/reference/execution-order.md` | `examples/async-fifo` | `api/vvm/timing/index.html` | Same-time ordering, ownership, and observable execution order need explicit documentation | Add clock and execution-order chapters |
| `TimedDut`, `TimingScheduler`, `TimingEvent`, `TimingRun`, timing errors | rustdoc, timed UART example, README | Medium | Accurate | Advanced users | `docs/book/src/guide/timing-models.md`, `docs/book/src/api-guide/schedulers.md`, `docs/book/src/reference/limitations.md` | `examples/timed-uart` | `api/vvm/timing/index.html` | Timing limitations, manual stepping, run-to-idle behavior, and separation from cycle scheduling need deeper explanation | Add timing chapters |
| `InoutState` and generated inout APIs | rustdoc, tri-state example, README | Medium | Accurate | Advanced users | `docs/book/src/guide/bidirectional-ports.md`, `docs/book/src/api-guide/inout.md`, `docs/book/src/reference/generated-types.md` | `examples/tri-state-bus` | `api/vvm/dut/index.html` | Contract, caller-owned resolution, floating policy, and two-state limitations need explicit guide | Add inout guide and reference |
| Coverage primitives: `Bin`, `Coverpoint`, `Cross2`, groups, sessions, merge, report | rustdoc, current coverage docs, example READMEs | Medium | Accurate | Users, CI maintainers | `docs/book/src/coverage/*.md`, `docs/book/src/api-guide/coverage.md`, `docs/book/src/cargo-vvm/*.md` | `examples/counter/src/coverage.rs`, `examples/timed-uart/src/lib.rs` | `api/vvm/coverage/index.html` | Needs progressive tutorial, design guidance, schema, merge compatibility, reporting, and CLI integration | Expand coverage and CLI sections |
| Packed values and generated type mapping | rustdoc, fixtures, current user-guide sentence | Low | Accurate | Users | `docs/book/src/reference/generated-types.md`, `docs/book/src/api-guide/drive-and-sample.md` | `tests/fixtures/native-packed-*`, `tests/fixtures/native-wide-transform` | `api/vvm/packed/index.html` | No consolidated mapping guide for scalars, wide values, enums, structs, arrays, or unsupported shapes | Add generated-types reference |
| Environment variables: `VVM_REPLAY`, `VVM_SEED`, `VVM_CYCLES`, `VVM_TRACE_DIR`, `VVM_COVERAGE_DIR`, Verilator selection | README, harness behavior, build docs | Medium | Accurate | Users | `docs/book/src/reference/environment-variables.md`, `docs/book/src/reference/configuration.md` | `examples/counter`, `examples/sync-fifo` | `api/vvm/test/struct.TestRunConfig.html`, `api/vvm_build/index.html` | Missing accepted formats, defaults, precedence, and errors in one table | Add environment-variable reference |
| Contributor command surface | `README.md`, `justfile`, current contributing chapter, `docs/dev/testing-strategy.md` | Medium | Accurate | Contributors | `docs/book/src/development/contributing.md`, `docs/book/src/development/common-commands.md`, `docs/book/src/development/testing.md` | `justfile` | n/a | Command matrix, focused workflows, and CI-equivalent flow need a full guide | Expand contributor docs |
| Maintainer strategy docs | `docs/dev/testing-strategy.md`, `docs/dev/example-strategy.md`, `docs/dev/rustdoc_style.md`, `docs/dev/public-api.md`, `docs/dev/errors-and-diagnostics.md` | High | Accurate | Maintainers | remain in `docs/dev/` with book links from development chapters | n/a | n/a | Need better discoverability from the book and contribution guide | Link from development chapters and docs-maintenance policy |

## README-Only Or Example-Only Information To Migrate

These topics are currently richer outside the book and must exist in book form
after the rewrite:

- Root README feature inventory and motivation.
- Root README timing-mode explanation and scheduler separation.
- Root README inout explanation and two-state warning.
- Root README functional-coverage capture, merge, report, and GitLab metric flow.
- Root README environment-variable list and replay precedence.
- Root README workspace crate inventory.
- Root README contributor setup and common commands.
- Counter README intentional-failure workflow and coverage-artifact workflow.
- Sync FIFO README queue-model semantics and coverage dimensions.
- Timed UART README timing semantics and manual coverage framing.
- Async FIFO README same-time ordering and direct multi-clock stepping.
- Tri-state README generated split-port contract, truth table, and settling loop.

## Public APIs Still Under-Explained Outside Rustdoc

- `vvm::prelude` and canonical import guidance.
- `vvm::include_dut!` naming and generated wrapper layout.
- `DutBuilder` normal workflow and failure stages.
- Derive and attribute macro usage rules and typical diagnostics.
- `Testbench` builder phases, ownership, and finalization behavior.
- `TestRunConfig` and `TestContext` capability-driven configuration.
- `ClockScheduler` versus `TimingScheduler` and why they stay separate.
- `InoutState` and caller-owned resolution semantics.
- `CoverageMergePolicy`, report detail controls, provenance, and fingerprints.
- Packed and aggregate generated type mappings.

## Weak Navigation And Terminology Gaps

- The current book offers no hierarchy between concepts, tutorials, API usage,
  reference material, and development material.
- Important terms such as transaction, sequence, observed cycle, timing slot,
  and generated DUT wrapper are used before they are fully explained.
- There is no single authoritative limitations chapter; limitations are split
  across the README, example READMEs, and book summaries.
- The current example chapter does not tell readers what files to study or what
  to read next.

## Page-By-Page Public Book Inventory

The following inventory records the public book state validated during the final
user-centred rewrite pass. Depth labels are relative to the final milestone 12.5
 goals: `substantial`, `partial`, or `sparse`.

### Orientation And Landing Pages

| Page | Intended audience | Current purpose | Current depth | Example dependence | Internal-process leakage | Missing background | Missing code | Missing diagnostics | Planned action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `introduction.md` | New users | Brief product overview | Partial | Low | None | Why VVM exists, ecosystem fit, and learning path sequencing are under-explained | n/a | Alpha limitations are listed but not connected to usage choices | Keep concise, but pair with dedicated Why/Fit/How chapters |
| `installation.md` | New users, contributors | Prerequisites and setup | Partial | Low | None | Cargo and Verilator roles need clearer HDL-user framing | Low | First-installation failure guidance needs stronger linkage | Expand targeted explanations and cross-links |
| `quick-start.md` | First-time users | First end-to-end workflow | Partial | High | None | Independent project creation, generated-file model, and runtime phases are missing | High; key code is elided or delegated to `examples/counter` | Result interpretation and trace troubleshooting are too narrow | Replace with independent tested fixture and complete source-backed walkthrough |
| `getting-started.md` | New users following old URL | Compatibility landing page | Sparse | Medium | Yes; mentions milestone split and old structure | Learning path value, prerequisites, and outcomes are not taught | n/a | n/a | Repurpose into useful landing page without implementation history |
| `user-guide.md` | Users entering the guide section | Guide overview | Sparse | Low | None | Section purpose and learning progression are under-explained | n/a | n/a | Rewrite as a useful guide landing page or reduce to compatibility pointer |
| `coverage.md` | Users discovering coverage | Coverage section overview | Partial | Medium | None | Coverage mental model and when to adopt it are too brief | Low | Failure and artifact expectations are brief | Expand as landing page that frames the coverage section |
| `cargo-vvm.md` | CI users | cargo-vvm section overview | Partial | Medium | None | Why to use cargo-vvm versus direct commands is brief | n/a | Failure-semantics path is too terse | Expand as landing page tied to artifact workflow |
| `examples.md` | Users seeking case studies | Examples section overview | Partial | High | None | Examples-as-case-studies role versus guide role is not explicit | n/a | n/a | Reframe examples as supporting case studies |
| `reference.md` | Users seeking exact values | Reference overview | Sparse | Low | None | Section contract versus Guide/API Guide is not explicit | n/a | n/a | Rewrite as section contract and index |

### Concepts

| Page | Intended audience | Current purpose | Current depth | Example dependence | Internal-process leakage | Missing background | Missing code | Missing diagnostics | Planned action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `concepts/overview.md` | New users | Concepts landing page | Sparse | Low | None | Concepts section role is too brief | n/a | n/a | Expand as a true conceptual map |
| `concepts/verification-workflow.md` | New users | High-level verification cycle | Partial | Medium | None | Runtime phases are present but not yet introduced as the book's core mental model | Low | Low | Promote to authoritative runtime mental-model chapter |
| `concepts/dut-and-wrapper.md` | New users | DUT wrapper concept | Partial | Medium | None | Build-time generation and wrapper ownership are too brief | Low | Low | Expand and cross-link to build/inclusion guides |
| `concepts/transactions.md` | HDL users new to Rust | Stimulus and observation concepts | Partial | Medium | None | Mapping between semantic transactions and raw ports needs stronger explanation | Low | Low | Expand with clearer HDL-oriented framing |
| `concepts/sequences.md` | Users writing tests | Sequence concept | Sparse | Medium | None | Iterator concept for HDL users is too brief | Low | Low | Expand around deterministic stimulus streams |
| `concepts/reference-models.md` | Verification engineers | Model concept | Sparse | Medium | None | Model ownership and abstraction boundary need more explanation | Low | Low | Expand with workflow-centered explanation |
| `concepts/scoreboards.md` | Verification engineers | Comparison concept | Sparse | Medium | None | Failure retention and scoreboard policy are under-explained | Low | Low | Expand with mismatch and policy framing |
| `concepts/clocks-and-time.md` | Users entering advanced timing | Clock and time concepts | Partial | Medium | None | Cycle-driven versus event-driven split needs stronger explanation | Low | Low | Expand and align with runtime chapters |
| `concepts/failures-and-results.md` | Users reading reports | Result model | Partial | Medium | None | What failures look like in practice is only lightly taught | Low | Medium | Expand using real result/report interpretation |

### Guide Chapters

| Page | Intended audience | Current purpose | Current depth | Example dependence | Internal-process leakage | Missing background | Missing code | Missing diagnostics | Planned action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `guide/project-setup.md` | New users | Crate layout and dependency setup | Sparse | Low | None | Cargo roles, crate choice, generated-output behavior, and rebuild tracking are missing | High | High | Rewrite as full setup chapter |
| `guide/build-script.md` | New users | `build.rs` workflow | Sparse | Medium | None | What Cargo runs and what `DutBuilder` owns are under-explained | High | High | Rewrite with declarative patterns and options |
| `guide/including-the-dut.md` | New users | Generated wrapper inclusion | Sparse | Medium | None | `OUT_DIR`, naming, and debug workflow are under-explained | High | High | Rewrite with generalized code and errors |
| `guide/driving-inputs.md` | Users writing stimulus | `Drive` usage | Sparse | Medium | None | Semantic transactions versus raw ports is under-explained | High | High | Rewrite with generalized transactions and compile-time diagnostics |
| `guide/sampling-outputs.md` | Users writing observations | `Sample` usage | Sparse | Medium | None | Observation semantics and raw-versus-derived data are under-explained | High | High | Rewrite with generalized observation examples |
| `guide/creating-a-testbench.md` | Users assembling runs | `Testbench` workflow | Sparse | High | None | Ownership, typestate progression, and failure lifecycle are under-explained | High | High | Rewrite as central chapter with full runtime cycle |
| `guide/registering-tests.md` | Users exposing tests to Cargo | `#[vvm::test]` usage | Sparse | Medium | None | Relationship to `#[test]`, nextest, capabilities, and filtering are under-explained | High | High | Rewrite with complete generalized tests |
| `guide/configuring-tests.md` | Users configuring execution | Runtime config | Sparse | Medium | None | Descriptor defaults, env precedence, and reproducibility need more context | High | High | Rewrite with concrete invocations and environment tables |
| `guide/randomization-and-replay.md` | Intermediate users | Deterministic pseudo-random workflow | Partial | High | None | Replay guarantees and sequence-design advice are too brief | Medium | Medium | Rewrite with generalized randomized stream and CI reproduction flow |
| `guide/waveform-tracing.md` | Intermediate users | Trace workflow | Sparse | Medium | None | Build-time enablement versus run-time enablement is too brief | Medium | Medium | Rewrite with lifecycle, performance, and troubleshooting |
| `guide/multi-clock.md` | Advanced users | Multi-clock execution | Sparse | High | None | Deterministic same-time ordering and scheduler guarantees are under-explained | Medium | Medium | Rewrite with generalized dual-domain pattern before async FIFO case study |
| `guide/timing-models.md` | Advanced users | Event-driven timing mode | Sparse | High | None | TimedDut, coroutine requirement, and timing limits are under-explained | Medium | Medium | Rewrite with generalized delayed-output fixture |
| `guide/bidirectional-ports.md` | Advanced users | Inout workflow | Sparse | High | None | Caller-owned resolution policy and two-state implications are under-explained | Medium | Medium | Rewrite with generalized line-resolution example before tri-state case study |
| `guide/troubleshooting.md` | All users | Common failure recovery | Partial | Medium | None | Failures are listed but not always linked to lifecycle phase or likely cause | Low | Medium | Expand around build, run, trace, timing, inout, and coverage diagnostics |

### API Guide

| Page | Intended audience | Current purpose | Current depth | Example dependence | Internal-process leakage | Missing background | Missing code | Missing diagnostics | Planned action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `api-guide/overview.md` | Advanced users | API guide landing page | Sparse | Low | None | Section contract versus Guide and Reference is not explicit | n/a | n/a | Expand as section map |
| `api-guide/facade-and-prelude.md` | Users mapping modules | Facade imports | Partial | Medium | None | Canonical import strategy and crate boundaries need stronger workflow framing | Low | Low | Expand and align with README surfaces |
| `api-guide/dut-builder.md` | Build-script authors | `DutBuilder` API summary | Partial | Medium | None | Option interactions and output effects need clearer tables | Medium | Medium | Expand and align with build guide |
| `api-guide/drive-and-sample.md` | Users of derives | Derive API summary | Partial | Medium | None | Supported shapes and common macro diagnostics are too brief | Medium | Medium | Expand and coordinate with guide chapters |
| `api-guide/clocks.md` | Users of clock helpers | Clock API summary | Partial | Medium | None | Single-clock versus scheduler workflow needs clearer separation | Low | Low | Expand with workflow map |
| `api-guide/testbench.md` | Users of runtime builder | Testbench API summary | Partial | Medium | None | Run methods, observers, and result types need fuller context | Low | Medium | Expand after guide rewrite |
| `api-guide/models-and-scoreboards.md` | Advanced users | Model and scoreboard API summary | Sparse | Medium | None | Trait contracts and common patterns are too brief | Low | Medium | Expand and link exact rustdoc |
| `api-guide/test-attribute.md` | Users of `#[vvm::test]` | Attribute syntax summary | Partial | Medium | None | Accepted signatures and capability effects are still terse | Medium | Medium | Expand after guide rewrite |
| `api-guide/configuration-and-context.md` | Users of config/context APIs | Config API summary | Sparse | Medium | None | `TestRunConfig` and `TestContext` roles need stronger differentiation | Medium | Medium | Expand with lifecycle tables |
| `api-guide/randomization.md` | Advanced users | Random API summary | Partial | Medium | None | Sequence contracts and replay reconstruction are brief | Medium | Medium | Expand and align with guide |
| `api-guide/tracing.md` | Advanced users | Trace API summary | Partial | Medium | None | Trace-capable DUT contract and report interaction are too brief | Medium | Medium | Expand |
| `api-guide/schedulers.md` | Advanced users | Timing scheduler summary | Sparse | Medium | None | Clock scheduler versus timing scheduler split needs clearer coverage | Medium | Medium | Expand |
| `api-guide/inout.md` | Advanced users | Inout API summary | Partial | High | None | Generated accessors and resolution contract need clearer general explanation | Medium | Medium | Expand and pair with generalized fixture |
| `api-guide/coverage.md` | Users of coverage APIs | Coverage API summary | Partial | Medium | None | Manual versus derived coverage and CLI relationship are brief | Medium | Medium | Expand |
| `api-guide/reports.md` | Users reading output | Reporting APIs | Sparse | Low | None | Result reporting surfaces are too brief | Low | Medium | Expand |
| `api-guide/errors.md` | Advanced users | Public error taxonomy | Partial | Low | None | Error grouping by lifecycle phase is missing | Low | Medium | Expand |

### Coverage, Examples, cargo-vvm, And Reference Pages

| Page | Intended audience | Current purpose | Current depth | Example dependence | Internal-process leakage | Missing background | Missing code | Missing diagnostics | Planned action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `coverage/bins.md` | Coverage authors | Bin design | Partial | High | None | General design advice is overshadowed by example framing | Medium | Medium | Generalize, then keep examples as case studies |
| `coverage/coverpoints.md` | Coverage authors | Coverpoint construction | Sparse | Medium | None | Mental model and practical heuristics are too brief | Medium | Medium | Expand |
| `coverage/typed-models.md` | Coverage authors | Derive-based coverage | Sparse | Medium | None | Supported shapes and naming strategy need detail | Medium | Medium | Expand |
| `coverage/crosses.md` | Coverage authors | Cross coverage | Sparse | Medium | None | Why and when to cross is too brief | Medium | Medium | Expand |
| `coverage/sampling.md` | Coverage authors | Sampling lifecycle | Sparse | Medium | None | Lifecycle hooks and failure behavior need fuller treatment | Medium | Medium | Expand |
| `coverage/sessions-and-artifacts.md` | CI users | Artifact persistence | Sparse | Medium | None | Session identity and file roles are under-explained | Low | Medium | Expand |
| `coverage/schema.md` | Advanced users | File schema reference | Sparse | Low | None | Exact field meanings are brief | Low | Low | Expand as reference |
| `coverage/merging.md` | CI users | Merge behavior | Sparse | Medium | None | Merge compatibility and failure cases need more explanation | Low | Medium | Expand |
| `coverage/reporting.md` | CI users | Reports | Sparse | Medium | None | Human versus machine outputs are brief | Low | Medium | Expand |
| `coverage/ci.md` | CI users | CI workflow | Sparse | High | None | End-to-end artifact flow is too brief | Low | Medium | Expand |
| `examples/counter.md` | New users | Counter case study | Partial | High | None | Must become supporting case study rather than primary tutorial | Low | Low | Retain as case study after Quick Start rewrite |
| `examples/sync-fifo.md` | Intermediate users | FIFO case study | Partial | High | None | Stronger mapping to guide concepts is needed | Low | Low | Tighten case-study role |
| `examples/timed-uart.md` | Advanced users | Timed UART case study | Partial | High | None | Needs clearer link back to timing concepts | Low | Low | Tighten case-study role |
| `examples/async-fifo.md` | Advanced users | Async FIFO case study | Partial | High | None | Needs clearer link back to multi-clock concepts | Low | Low | Tighten case-study role |
| `examples/tri-state-bus.md` | Advanced users | Inout case study | Partial | High | None | Needs clearer link back to generalized inout explanation | Low | Low | Tighten case-study role |
| `cargo-vvm/installation.md` | CI users | Install command | Partial | Low | None | Workflow context is brief | n/a | Low | Expand slightly |
| `cargo-vvm/workflow.md` | CI users | Coverage workflow | Partial | Medium | None | Output stages and failure propagation need more context | Low | Medium | Expand |
| `cargo-vvm/command-reference.md` | CI users | Exact CLI syntax | Partial | Low | None | Flags are present but not always motivated | n/a | Low | Expand reference framing |
| `cargo-vvm/output-layout.md` | CI users | Artifact layout | Partial | Low | None | File roles are too brief | n/a | Low | Expand |
| `cargo-vvm/merge-policies.md` | CI users | Merge rules | Partial | Low | None | Real decision guidance is brief | n/a | Low | Expand |
| `cargo-vvm/failure-semantics.md` | CI users | Exit-code behavior | Partial | Low | None | CI interpretation examples are brief | n/a | Low | Expand |
| `cargo-vvm/gitlab.md` | GitLab CI users | GitLab recipe | Partial | Medium | None | How it relates to generic workflow needs more framing | Low | Low | Keep but better contextualize |
| `cargo-vvm/troubleshooting.md` | CI users | CLI troubleshooting | Partial | Medium | None | Symptom-to-cause mapping is brief | Low | Medium | Expand |
| `reference/configuration.md` | Users seeking exact config values | Configuration defaults | Sparse | Low | None | Descriptor/runtime/environment split is incomplete | Low | Medium | Expand tables |
| `reference/environment-variables.md` | Users seeking exact env values | Environment-variable reference | Sparse | Low | None | Accepted values, precedence, and invalid forms are incomplete | Low | Medium | Expand tables |
| `reference/generated-types.md` | Users mapping HDL to Rust | Type mapping reference | Partial | Medium | None | Mapping completeness and unsupported shapes are incomplete | Medium | Medium | Expand with fixture-backed tables |
| `reference/execution-order.md` | Advanced users | Observable execution order | Sparse | Low | None | Cycle versus timing order and failure ordering need exact detail | Low | Medium | Expand |
| `reference/artifact-layout.md` | Users locating outputs | Artifact layout | Sparse | Low | None | Book, API, trace, and coverage artifact locations need fuller coverage | Low | Low | Expand |
| `reference/compatibility.md` | Users checking support | Compatibility summary | Sparse | Low | None | Must defer to one authoritative limitations/support page | n/a | n/a | Merge into stronger limitations/support path |
| `reference/diagnostics.md` | Users reading failures | Diagnostic output reference | Sparse | Low | None | Build/test/report diagnostic shapes are brief | Low | Medium | Expand |
| `reference/terminology.md` | New and advanced users | Canonical term list | Sparse | Low | None | Several key terms need stronger first-use definitions in chapters too | n/a | n/a | Expand and cross-link |
| `reference/limitations.md` | All users | Central limitations page | Sparse | Low | None | Good direction, but support policy and exact limits are too brief | n/a | Medium | Expand into authoritative support-and-limitations page |

## Future Information Architecture

The rewrite will move the book to this section-level structure:

- `introduction.md`
- `quick-start.md`
- `installation.md`
- `concepts/`
- `guide/`
- `api-guide/`
- `coverage/`
- `cargo-vvm/`
- `examples/`
- `reference/`
- `development/`
- `404.md`

Existing published one-file pages will be replaced by:

- hierarchical content at the new paths;
- pointer chapters where a short compatibility page remains useful;
- updated README and in-book cross-links to the new structure.

## Task Mapping

- `TASK-18`: audit and rewrite plan.
- `TASK-19`: mdBook information architecture.
- `TASK-20`: root and package READMEs.
- `TASK-21`: introduction, installation, and quick start.
- `TASK-12`: concepts and core workflow.
- `TASK-22`: public API usage guide.
- `TASK-4`: advanced execution guides.
- `TASK-5`: functional coverage guide.
- `TASK-6`: `cargo-vvm` guide.
- `TASK-7`: curated example chapters.
- `TASK-23`: practical reference material.
- `TASK-8`: developer architecture.
- `TASK-13`: contributor and maintenance documentation.
- `TASK-24`: final validation and milestone closure.

## Rewrite Plan

1. Replace the flat book navigation with a hierarchical `SUMMARY.md` and new
   directories.
2. Build the orientation path first: introduction, installation, quick start,
   and concept chapters.
3. Build the workflow and API guide around the actual facade modules and tested
   example code.
4. Expand advanced execution, coverage, `cargo-vvm`, example, reference, and
   development sections until README-only knowledge is eliminated.
5. Rewrite the root and package READMEs to match the new book and keep them
   useful in isolation.
6. Run documentation commands, package-doc checks, curated example workflows,
   and full repository validation.
7. Only then finalize all milestone tasks and close milestone 12.5.

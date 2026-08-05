# VVM Agent Guide

## Workspace

- Rust 2024 workspace (MSRV 1.87) for Verilator-backed verification; building the example crates and native integration requires Verilator plus a C++ toolchain.
- `crates/vvm` is the public facade (`vvm-rs` package, `vvm` library); `vvm-core` owns runtime/testbench primitives, `vvm-build` runs Verilator and generates/compiles bridges from consumer `build.rs`, and `vvm-macros` owns derives and `#[vvm::test]`.
- `examples/README.md` is the public learning ladder: counter, synchronous FIFO, timed UART, asynchronous FIFO, and tri-state bus. Native generated-port regressions live under `tests/fixtures/native-*`; see `docs/dev/example-strategy.md`.
- Generated DUT source is included from `OUT_DIR`; keep `DutBuilder::new("name")` and `vvm::include_dut!(name)` aligned.

## Commands

- Prefer `just` recipes.
- Run `just init` once to install contributor tooling, including nightly Rust, nextest, coverage, udeps, audit, Markdown TOC, and pre-commit.
- Format with `just fmt` or check without edits using `just fmt --check`; formatting explicitly uses `cargo +nightly fmt --all`.
- Run linting with `just check -- -D warnings`; it checks all workspace targets, tests, examples, and features.
- Use `just test-fast` for pure-Rust unit, integration, UI, and fixture tests; use `just test-examples`, `just test-native-fixtures`, `just test-native`, `just test-e2e`, and `just test-package` for their corresponding boundaries. `just test-all` runs every category. The authoritative placement and execution policy is `docs/dev/testing-strategy.md`.
- Use `just benchmark`, `just benchmark-save-baseline NAME=<name>`, `just benchmark-compare-baseline NAME=<name>`, and `just benchmark-target <package> <target>` for local Criterion benchmarking. Baselines live under `target/criterion` and are machine-specific.
- `just ci` is the CI-equivalent, non-mutating verification: format check, lint with warnings denied, `cargo +nightly udeps`, audit, every test category, doctests, and coverage. It writes coverage reports under `coverage/`.
- Benchmarks are local-only developer workflows. Do not add them to `just ci`, pre-commit, or GitLab CI.
- The pre-commit hook runs `just ci` for Rust/TOML/justfile changes. README changes also run `just index`, which rewrites the README TOC.

## Tests And Fixtures

- VVM tests use normal Rust discovery. Unit-style VVM tests must be in `#[cfg(test)]`; integration tests under `tests/` are already test-only.
- The `vvm-macros` compile-test suite uses `trybuild`; failure diagnostics in `crates/vvm-macros/tests/ui/fail/*.stderr` are expected snapshots and must be updated deliberately with macro diagnostic changes.
- `vvm-build` codegen and Verilator metadata fixtures under `crates/vvm-build/tests/fixtures/` are checked-in golden inputs/outputs. The metadata fixtures are versioned for Verilator 5.048; do not casually regenerate or reformat them.
- Replayable VVM tests can be reproduced with `VVM_REPLAY` (takes precedence over `VVM_SEED`); `VVM_CYCLES` and `VVM_TRACE_DIR` configure supported tests, with default trace output rooted at `target/vvm-trace/`.

## Rustdoc And Lints
- The root workspace enables strict `missing_docs` and `clippy::missing_docs_in_private_items` plus other strict code readability and bug-proning lints.
- When adding or rewriting docs, match `docs/dev/rustdoc_style.md` rather than the stale README link to `dev/docs/rustdoc_style.md`.

## Non-negotiable lint policy

Lint compliance must be achieved by correcting the implementation, not by suppressing diagnostics.

Never add, broaden, move, or modify any lint-suppression mechanism, including:

* `#[allow(...)]`
* `#![allow(...)]`
* `#[expect(...)]`
* `#![expect(...)]`
* `cfg_attr(..., allow(...))`
* `cfg_attr(..., expect(...))`
* `-A` or `--allow` compiler and Clippy arguments
* Cargo lint configuration that lowers an existing lint level

Existing suppressions are not precedent for adding new suppressions.

When a lint fires:

1. Understand why the lint considers the code problematic.
2. Refactor the implementation so that the diagnostic no longer applies.
3. Preserve the intended semantics and architecture.
4. Run the complete lint command again.

When a lint cannot be resolved cleanly without changing semantics or violating the architecture, stop and report:

* the complete diagnostic;
* the affected location;
* the refactorings attempted;
* why those approaches were unsuitable;
* the smallest design decision needed from the user.

Do not suppress the lint and do not describe the task as complete.

Only exception to this rule is condintional include of dead code reserved for future use/features. E.g.:
```
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Reserved for structured runtime diagnostics.")
)]
fn diagnose_timing(...

```

## Readability and abstraction

Optimize for maintainability and clarity, not minimum line count.

Prefer:

* descriptive intermediate variables over dense expressions;
* named lifecycle operations over inline implementation details;
* explicit state types over using `Option` or boolean values as an implicit state machine;
* one visually separated block per logical phase;
* centralized cleanup over repeated cleanup calls in multiple branches.

As a review target, nontrivial functions should normally remain below 50 lines. Exceeding that target requires checking whether lifecycle phases or domain operations should be extracted.

Do not create tiny helpers that merely rename a single expression. Extract helpers that establish meaningful abstraction boundaries.

## Visual paragraphing

Use blank lines to divide function bodies into logical paragraphs. This is mandatory formatting, not an optional stylistic preference.

A nontrivial function must not be written as one uninterrupted sequence of statements.

Insert exactly one blank line:

* after an early-return guard clause;
* between input acquisition and validation;
* between validation and state mutation;
* between separate lifecycle phases;
* before committing calculated state after fallible operations succeed;
* before the final result construction or return expression when it represents a distinct phase;
* between independent checked calculations;
* between initialization, execution, and finalization phases.

Keep statements together only when they form one tightly coupled operation, such as:

* constructing one value;
* destructuring one value;
* updating several fields as one state commit;
* a method chain implementing one query;
* calculating closely related arguments for the immediately following operation.

### Required style

```rust
if !self.is_initialized() {
    return Err(TimingSchedulerError::NotInitialized);
}

let current = dut.simulation_time();
let pending = dut
    .events_pending()
    .map_err(|source| TimingSchedulerError::Dut {
        stage: TimingStage::EventsPending,
        time: current,
        source,
    })?;

if !pending {
    return Ok(None);
}

let next = dut
    .next_time_slot()
    .map_err(|source| TimingSchedulerError::Dut {
        stage: TimingStage::NextTimeSlot,
        time: current,
        source,
    })?
    .ok_or(TimingSchedulerError::MissingTimeSlot { time: current })?;

let elapsed = event_delta::<D::Error>(current, next)?;
let next_statistics = self.next_statistics::<D::Error>(current)?;

Dut::advance_time(dut, elapsed).map_err(|source| TimingSchedulerError::Dut {
    stage: TimingStage::AdvanceTime,
    time: current,
    source,
})?;

Dut::evaluate(dut).map_err(|source| TimingSchedulerError::Dut {
    stage: TimingStage::EvaluateTimeSlot,
    time: next,
    source,
})?;

let event = TimingEvent::new(self.time_slots, next, elapsed);

self.time_slots = next_statistics.time_slots;
self.evaluations = next_statistics.evaluations;

Ok(Some(event))
```

### Forbidden style

Do not collapse all phases into a continuous sequence merely because the code compiles and `rustfmt` accepts it:

```rust
if !self.is_initialized() {
    return Err(TimingSchedulerError::NotInitialized);
}
let current = dut.simulation_time();
let pending = dut.events_pending()?;
if !pending {
    return Ok(None);
}
let next = dut.next_time_slot()?;
let elapsed = event_delta(current, next)?;
Dut::advance_time(dut, elapsed)?;
Dut::evaluate(dut)?;
self.time_slots += 1;
self.evaluations += 1;
Ok(Some(event))
```

## Required self-review

Before completing a task:

1. Inspect the entire diff for lint suppressions.
2. Inspect changed functions for mixed abstraction levels.
3. Refactor functions whose control flow requires tracking several unrelated responsibilities simultaneously.
4. Run formatting, linting, tests, and the repository’s lint-suppression check.
5. Report any remaining readability trade-offs explicitly.
6. Perform a visual-paragraphing pass over every changed function.

<!-- BACKLOG.MD MCP GUIDELINES START -->

<CRITICAL_INSTRUCTION>

## BACKLOG WORKFLOW INSTRUCTIONS

This project uses Backlog.md MCP for all task and project management activities.

**CRITICAL GUIDANCE**

- If your client supports MCP resources, read `backlog://workflow/overview` to understand when and how to use Backlog for this project.
- If your client only supports tools or the above request fails, call `backlog.get_workflow_overview()` tool to load the tool-oriented overview (it lists the matching guide tools).

- **First time working here?** Read the overview resource IMMEDIATELY to learn the workflow
- **Already familiar?** You should have the overview cached ("## Backlog.md Overview (MCP)")
- **When to read it**: BEFORE creating tasks, or when you're unsure whether to track work

These guides cover:
- Decision framework for when to create tasks
- Search-first workflow to avoid duplicates
- Links to detailed guides for task creation, execution, and finalization
- MCP tools reference

You MUST read the overview resource to understand the complete workflow. The information is NOT summarized here.

</CRITICAL_INSTRUCTION>

<!-- BACKLOG.MD MCP GUIDELINES END -->

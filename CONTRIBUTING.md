# Contributing

Thanks for contributing to VVM.

## Requirements

- Rust `1.87.0` or the current stable Rust toolchain.
- Cargo.
- Verilator.
- A Linux C++ toolchain.

Ordinary native models use a C++17-capable compiler. Timing-enabled models
additionally require a C++20 compiler with coroutine support.

## First-Time Setup

Install `just` if needed:

```bash
cargo install just
```

Then run the bootstrap recipe from the repository root:

```bash
just init
```

`just init` installs or checks the tools used by repository validation,
including nightly `rustfmt`, `cargo-nextest`, `cargo-llvm-cov`,
`cargo-udeps`, `cargo-audit`, `cargo-deny`, `cargo-public-api`, `mdbook`, Markdown TOC,
and pre-commit hooks.

The `justfile` uses `dotenv-load := true`, so recipes automatically load a
local `.env` file when one exists.

## Workflow

Use Backlog.md for non-trivial work.

- Search for an existing task before creating a new one.
- Create or update a task when the work requires planning or design choices.
- Keep the implementation plan, notes, validation evidence, and final summary in
  the Backlog task.
- Use normal repository edits only for code and docs; do not hand-edit Backlog
  markdown files.

Prefer `just` recipes over ad hoc commands whenever a recipe exists.

## Common Commands

| Command | Scope |
| --- | --- |
| `just fmt` | format Rust sources |
| `just fmt --check` | verify formatting without edits |
| `just check -- -D warnings` | run workspace Clippy with warnings denied |
| `just test-fast` | pure-Rust unit, integration, UI, and fixture coverage |
| `just test-native` | Verilator-backed tests, native fixtures, and examples |
| `just test-e2e` | release-facing example and coverage workflows |
| `just test-package` | packaged-crate and packaged-consumer validation |
| `just ci` | repository CI-equivalent validation |
| `just docs` | assemble the published documentation site |
| `just api-diff` | compare the current public API against `docs/dev/api/0.2.0` |
| `just deny` | run dependency advisory, license, ban, and source policy checks |
| `just repro-check` | run focused deterministic and isolated reproducibility checks |
| `just release-audit` | run the full pre-RC release audit and retain audit logs |

Run `just --list` for the full recipe list.

## Validation Expectations

Match validation depth to the change.

- Run the smallest focused test or fixture that proves the contract you changed.
- Run `just fmt` or `just fmt --check` before finalizing Rust changes.
- Run `just check -- -D warnings` for implementation changes.
- Run `just deny` when dependency or release-policy changes affect advisories, licenses, bans, or sources.
- Run `just repro-check` when codegen, packaging, documentation assembly, or release-facing reproducibility behavior changes.
- Run `just docs` or at least `mdbook build docs/book` for book-content changes.
- Run `just test-package` for package metadata, package layout, packaged fixtures,
  or release-facing workflow changes.
- Run `just ci` before merging broad or release-facing work when feasible.

## FFI And Codegen Expectations

- Treat `docs/dev/ffi-safety-audit.md` as the current inventory of the generated
  Rust/C++/Verilator boundary.
- Preserve the generated wrapper ownership, pinning, finalization, trace, and
  transfer-length invariants when changing codegen or runtime boundary code.
- Generated DUT wrappers are intentionally thread-confined and are not part of a
  supported `Send` or `Sync` surface.
- Do not let a Rust panic or C++ exception cross an unsupported boundary.

Do not suppress lint failures to make the gate pass. Fix the implementation.

## Testing Layers

The repository deliberately separates validation into layers.

- Unit tests cover private invariants.
- Public crate integration tests cover supported facade and tooling contracts.
- Macro UI tests cover compile-time diagnostics.
- Fixture workspaces cover isolated consumer scenarios.
- Native fixtures cover generated-wrapper regressions and public DUT contracts.
- Examples act as both documentation and maintained end-to-end workflows.

Use the lowest layer that proves the behavior you changed.

## Documentation Expectations

- Keep public docs, crate READMEs, and rustdoc aligned with actual behavior.
- Follow `docs/dev/rustdoc_style.md` for rustdoc changes.
- Keep the compatibility matrix truthful. Do not expand platform, toolchain, or
  version claims without adding validation coverage.
- Update `CHANGELOG.md` for user-visible behavior, workflow, compatibility, or
  documentation changes that matter to downstream users or maintainers.

## Package Validation Expectations

Release-facing changes must preserve the publication shape.

- Keep publishable crate metadata, READMEs, and included support files accurate.
- Validate package archives with `just test-package`.
- When changing packaged resources, confirm extracted crate contents still build
  and the packaged consumer workflows still pass.
- Do not add workspace-only assumptions to publishable crates.

## Public API Review Expectations

The `v0.2.0` release line treats `docs/dev/api/0.2.0` as the accepted public API
baseline.

- Run `just api-diff` when changing public Rust APIs.
- Review `docs/dev/public-api.md` before adding exports or changing canonical
  module paths.
- Treat `vvm::__private` as unsupported generated-source ABI, not as user API.
- If a breaking public change is intentional, document it in the changelog and
  update the accepted baseline only after review.

## Pre-Commit Hooks

Install hooks with either:

```bash
just install-hooks
```

or:

```bash
pre-commit install
```

The configured hooks run these project-specific checks:

| Changed files | Hook behavior |
| --- | --- |
| `*.rs`, `*.toml`, or `justfile` | Runs `just ci`. |
| `README.md` | Runs `just index`, which rewrites the README table of contents. |
| Commit messages | Checks Conventional Commits format. |

## Commit Messages

Commit messages are checked by the `conventional-commit-check` hook. Use
Conventional Commits-style messages such as:

```text
docs(macro): documented Sample derive macro
fix(build): DutBuilder now rejects invalid verilator configuration
test: added public API integration tests
maint: cleaned up superfluous just recipes
```

## Benchmarking

Benchmarks are local-only developer tools.

- Use `just benchmark`, `just benchmark-save-baseline <name>`,
  `just benchmark-compare-baseline <name>`, and focused `just benchmark-target`
  runs.
- Criterion baselines live under `target/criterion` and are machine-specific.
- Benchmarks do not run in `just ci`, pre-commit hooks, or GitLab CI.

## Reporting Defects

Include the most relevant environment details when reporting an issue:

- operating system;
- Rust version;
- Verilator version;
- compiler used for native builds;
- reproduction steps, logs, and minimal examples when possible.

## Maintainer Release Workflow

### Release boundaries

- The intended publishable set is `vvm-core`, `vvm-macros`, `vvm-build`,
  `vvm-rs`, and `cargo-vvm`.
- Publish in dependency order: `vvm-core`, `vvm-macros`, `vvm-build`, `vvm-rs`,
  then `cargo-vvm`.

### Development-version progression

- Keep ordinary development off the exact published release commit.
- Before publishing, ensure every publishable crate version, internal dependency
  version, and release tag agree.
- After a final release, move the default branch to the next unreleased
  development version before resuming normal feature work.

### Dry-run validation before publishing

Run the release-facing validation before any production publish attempt:

```bash
just release-audit
just deny
just test-package
mdbook build docs/book
cargo package --list -p vvm-core
cargo package --list -p vvm-macros
cargo package --list -p vvm-build
cargo package --list -p vvm-rs
cargo package --list -p cargo-vvm
cargo publish --dry-run -p vvm-core
cargo publish --dry-run -p vvm-macros
cargo publish --dry-run -p vvm-build
cargo publish --dry-run -p vvm-rs
cargo publish --dry-run -p cargo-vvm
```

Review package file lists manually before publishing.

The release audit retains tool-version and command logs under
`target/vvm-release-audit/`.

### Protected credentials

- Production crates.io publication requires a protected `CARGO_REGISTRY_TOKEN`.
- Any GitLab token used for release tags, release objects, or protected manual
  publish jobs must also be protected and restricted to the intended release
  refs.
- Never expose production credentials to branch pipelines, merge requests, or
  unprotected tags.

### Publication sequence

1. Finalize the changelog and contributor-facing release notes.
2. Re-run the dry-run validation.
3. Publish crates in dependency order.
4. Wait for each crates.io upload to become available before publishing the next
   dependent crate.
5. Create the release tag only for the exact reviewed release commit.
6. Create the GitLab release and publish the matching documentation outputs.

### Partial-publication recovery

- If publication fails before any crate is uploaded, fix the issue and retry the
  same version only after re-running dry-run validation.
- If some crates are already published, do not try to overwrite them.
- Bump the unpublished crates to a new version, update internal dependency
  versions, and repeat the dry-run flow from the first unpublished dependency.
- Record the partial-publication state and the recovery plan in Backlog before
  continuing.

### Yanking policy

- Yank only when a published crate is materially broken or unsafe for normal
  consumption.
- Do not use yanks as a substitute for ordinary release correction.
- When yanking, document the reason in Backlog and in the next changelog entry,
  then publish the corrective follow-up release.

## License

Any contribution intentionally submitted for inclusion in this work shall be
dual licensed under Apache-2.0 and MIT, without additional terms or conditions.

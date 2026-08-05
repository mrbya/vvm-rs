# Contributing

## Requirements

- Rust 1.87.0 or newer.
- Cargo.
- Verilator.
- A working Linux C++ toolchain.

## First-Time Setup

nstall just if needed:

```bash
cargo install just
```

Then run the bootstrap recipe from the repository root:

```bash
just init
```

The bootstrap installs or checks the tools used by the project gates:

- nightly Rust
- cargo-binstall
- cargo-nextest
- cargo-llvm-cov
- cargo-udeps
- cargo-audit
- mdbook
- global markdown-toc
- pre-commit hooks

The justfile has set dotenv-load := true, so recipes automatically load a local .env file when one exists.

## Common Commands

| Command | Scope |
| --- | --- |
| `just fmt` | applies formatting to rust sources |
| `just fmt --check` | checks formatting |
| `just check` | clippy across the workspace |
| `just thorough-check` | checks formatting, audit, unused deps and more |
| `just test-all` | full test matrix |
| `just test-*` | focused test target |
| `just test-cov` | full test matrix with coverage report |
| `just docs` | assemble full docs book |
| `just docs-serve` | builds and locally serves full docs book |
| `just ci` | full repository gate |

For the full list of available just recipes run `just list` or `just help`.

Prefer `just` recipes over ad hoc Cargo commands whenever a recipe exists.

## CI Gate

`just ci` runs:

1. `just thorough-check`
2. `just test-all`
3. `just doctest`
4. `just test-cov-ci`

## Pre-Commit Hooks


Install hooks with:

```bash
just install-hooks
```

or:

```bash
pre-commit install
```

The configured hooks run these project-specific checks:

| Changed Files | Hook Behavior |
| --- | --- |
| `*.rs`, `*.toml`, or `justfile` outside bundled templates | Runs `just ci`. |
| `README.md` | Runs `just index`, which rewrites the README table of contents. |
| Commit messages | Checks Conventional Commits format. |

The hook config also includes standard whitespace, end-of-file, TOML, JSON, YAML, and large
file checks.

## Commit Messages

Commit messages are checked by the `conventional-commit-check` hook. Use
Conventional Commits-style messages, such as:

```text
docs(macro): documented Sample derive macro
fix(build): reject invalid verilator configuration
test: added public API integration tests
maint: cleaned up superfluous just recipes
```

## Documentation

Build full doc suite before submitting changes:

```bash
just docs
```

Check built docs with local serve:
```bash
just docs-serve
```

When adding or rewriting Rustdoc, follow the repository rustdoc style guidance
and keep the strict missing-doc lint policy intact.

## Testing

The repository testing policy is intentionally layered and enforced through the
`just` command surface.

At a high level, the repository separates:

- unit tests for private invariants;
- public crate integration tests;
- macro compile-time tests;
- fixture workspaces for isolated consumer scenarios;
- curated examples for end-to-end public workflows;
- native fixtures for generated-port regressions.

Use the lowest layer that proves the contract you changed.

## Examples

The curated examples are product documentation and regression targets at the same
time. They should stay readable, source-backed, and aligned with the public API.

Treat curated examples as product documentation and keep them aligned with the
published example ladder and example chapters.

## Benchmarking

Benchmark commands live in the `justfile` and route through Criterion only.

Useful commands:

- `just benchmark`
- `just benchmark-save-baseline NAME=local`
- `just benchmark-compare-baseline NAME=local`
- `just benchmark-target vvm-core packed`

Use the save-before-change and compare-after-change workflow for performance work:

```bash
just benchmark-save-baseline NAME=before-feature

# implement the feature or optimization

just benchmark-compare-baseline NAME=before-feature
```

Benchmarks are local-only developer tools.

- Criterion stores local data and baselines under `target/criterion`.
- Baselines are machine-specific and should not be compared casually across
  unrelated systems.
- `just ci`, GitLab CI, and pre-commit hooks do not run benchmarks.

Benchmarks are for performance investigation, not for proving correctness.

## Releases

Release-facing work in this repository includes:

- keeping package metadata aligned;
- preserving useful package READMEs for crates.io;
- validating package archives with `just test-package`;
- keeping documentation, examples, and compatibility claims consistent;
- updating `CHANGELOG.md`.

## Compatibility

Contributor-facing compatibility work means keeping the documented support claims
aligned with CI evidence. Do not change MSRV, Verilator support, or platform
claims casually.

If a compatibility claim changes, update the relevant CI coverage and the public
compatibility documentation together.

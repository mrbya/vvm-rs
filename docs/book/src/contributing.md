# Contributing

This page is the user-facing contributor entry point and preserves the long-lived
`contributing.html` path.

## First-time Setup

```bash
cargo install just
just init
```

`just init` installs the tools used by repository checks, including nightly
rustfmt, nextest, LLVM coverage, udeps, audit, mdBook, Markdown TOC, and
pre-commit.

## Common Commands

| Command | Purpose |
| --- | --- |
| `just fmt --check` | formatting check |
| `just check -- -D warnings` | lint all targets, examples, and tests |
| `just test-fast` | pure-Rust unit, integration, UI, and fixture suites |
| `just test-native` | native tests, examples, and native fixtures |
| `just test-all` | every test category |
| `just docs-test` | doctests plus mdBook build |
| `just docs-links` | assembled site plus local entry-point checks |
| `just ci` | repository CI-equivalent gate |

## Workflow Expectations

- Use curated examples for public learning workflows.
- Use `tests/fixtures/` for narrow generated-port and native regression cases.
- Keep docs, rustdoc, READMEs, examples, and compatibility claims aligned.
- Do not weaken lint or validation checks to make a documentation change pass.

Use the rest of the development section for the full contributor guide, command
surface, testing model, documentation maintenance policy, and release workflow.

# Installation

## Requirements

For normal VVM usage, install:

- Rust 1.87.0 or newer.
- Cargo.
- Verilator.
- A working Linux C++ toolchain.

Timing-enabled models additionally need compiler coroutine support.

VVM does not currently claim broad cross-platform native support. The tested
path is Linux-based development and CI.

## Add Dependencies To A Consumer Project

Most users depend on `vvm-rs` as `vvm` in test code and `vvm-build` in
`build.rs`:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", version = "0.1.0-alpha.1" }

[build-dependencies]
vvm-build = "0.1.0-alpha.1"
```

During pre-release evaluation you can also use a Git dependency when you need a
not-yet-published change:

```toml
[dev-dependencies]
vvm = { package = "vvm-rs", git = "https://gitlab.com/byacrates/vvm-rs.git" }

[build-dependencies]
vvm-build = { git = "https://gitlab.com/byacrates/vvm-rs.git" }
```

Install `cargo-vvm` separately when you want suite-level functional-coverage
merge and reports:

```bash
cargo install cargo-vvm
```

## Contributor Setup

The repository uses `just` recipes as the primary command surface. Install
`just`, then bootstrap the workspace once:

```bash
cargo install just
just init
```

`just init` installs the extra tooling used by this repository, including
nightly rustfmt, nextest, LLVM coverage, udeps, audit, mdBook, Markdown TOC,
and pre-commit hooks.

## Verify Your Toolchain

Useful checks before starting a new VVM project:

```bash
rustc --version
cargo --version
verilator --version
```

If Verilator is not on `PATH`, VVM can use an explicit executable path through
the `VERILATOR` environment variable during the build.

## Next Steps

- Follow [Quick Start](quick-start.md) for a complete first project.
- Use [Project Setup](guide/project-setup.md) when adapting the workflow to an
  existing repository.
